use crate::gpu::GpuContext;
use crate::math::{Rect, Size, Vec2};

use super::{GpuTexture, GpuTextureView, TextureFormat, TextureKind};
use parking_lot::Mutex;
use std::borrow::Cow;

#[derive(Debug)]
pub struct TextureAtlas<Key: AtlasKeySource>(Mutex<AtlasStorage<Key>>);

/*

v____   AtlasTextureId
_____________________________________
.      AtlasTile | TextureId        .
.     ........                      .
.     ........                      .
.     ........                      .
.     ........                      .
.                                   .
.                                   .
_____________________________________
*/

use std::fmt::Debug;
use std::hash::Hash;

pub trait AtlasKeySource: Hash + Debug + Clone + PartialEq + Eq {
    fn texture_kind(&self) -> TextureKind;
}

pub type AtlasTextureInfoMap<Key> = ahash::AHashMap<Key, AtlasTextureInfo>;

#[derive(Debug)]
struct AtlasStorage<Key: AtlasKeySource> {
    gpu: GpuContext,
    gray_textures: AtlasTextureList<Option<AtlasTexture>>,
    color_textures: AtlasTextureList<Option<AtlasTexture>>,
    key_to_tile: ahash::AHashMap<Key, AtlasTile>,
}

impl<Key: AtlasKeySource> TextureAtlas<Key> {
    pub fn new(gpu: GpuContext) -> Self {
        Self(Mutex::new(AtlasStorage::<Key> {
            gpu,
            gray_textures: Default::default(),
            color_textures: Default::default(),
            key_to_tile: ahash::AHashMap::new(),
        }))
    }

    pub fn get_texture_for_tile<R>(
        &self,
        tile: &AtlasTile,
        f: impl FnOnce(&AtlasTexture) -> R,
    ) -> Option<R> {
        let lock = self.0.lock();
        lock.with_texture(tile, f)
    }

    pub fn get_texture_for_key<R>(
        &self,
        key: &Key,
        f: impl FnOnce(&AtlasTexture) -> R,
    ) -> Option<R> {
        let lock = self.0.lock();
        let tile = lock.key_to_tile.get(key)?;
        lock.with_texture(tile, f)
    }

    pub fn get_texture_info(&self, key: &Key) -> Option<AtlasTextureInfo> {
        let lock = self.0.lock();
        lock.get_texture_info(key)
    }

    pub fn get_texture_infos(
        &self,
        keys: impl Iterator<Item = Key>,
        map: &mut AtlasTextureInfoMap<Key>,
    ) {
        let lock = self.0.lock();

        map.extend(
            keys.map(|id| (id.clone(), lock.get_texture_info(&id)))
                .filter_map(|(id, info)| info.map(|info| (id, info))),
        )
    }

    pub fn get_or_insert<'a>(
        &'a self,
        key: &'a Key,
        insert: impl FnOnce() -> (Size<i32>, Cow<'a, [u8]>),
    ) -> AtlasTile {
        let mut lock = self.0.lock();
        let tile = lock.key_to_tile.get(key);

        if let Some(tile) = tile {
            return tile.clone();
        }
        let (size, data) = insert();

        let tile = lock.create_texture(size, key.clone());
        lock.upload_texture(&tile, &data);
        tile
    }

    /// Combination of `create_texture` and `upload_texture`
    pub fn create_texture_init(&self, key: &Key, size: Size<i32>, data: &[u8]) -> AtlasTile {
        let mut lock = self.0.lock();
        let tile = lock.create_texture(size, key.clone());
        lock.upload_texture(&tile, data);
        tile
    }

    /// Allocates a tile of given size on an available texture slot and returns the tile
    /// use the `upload_texture` method to upload data into tile
    pub fn create_texture(&self, key: &Key, size: Size<i32>) -> AtlasTile {
        let mut lock = self.0.lock();
        lock.create_texture(size, key.clone())
    }

    pub fn upload_texture(&self, tile: &AtlasTile, data: &[u8]) {
        let lock = self.0.lock();
        lock.upload_texture(tile, data)
    }
}

impl<Key: AtlasKeySource> AtlasStorage<Key> {
    fn get_storage_write(
        &mut self,
        kind: &TextureKind,
    ) -> &mut AtlasTextureList<Option<AtlasTexture>> {
        match kind {
            TextureKind::Mask => &mut self.gray_textures,
            TextureKind::Color => &mut self.color_textures,
        }
    }

    fn get_storage_read(&self, kind: &TextureKind) -> &AtlasTextureList<Option<AtlasTexture>> {
        match kind {
            TextureKind::Mask => &self.gray_textures,
            TextureKind::Color => &self.color_textures,
        }
    }

    fn with_texture<R>(&self, tile: &AtlasTile, f: impl FnOnce(&AtlasTexture) -> R) -> Option<R> {
        let storage = self.get_storage_read(&tile.texture.kind);
        let texture = storage[tile.texture.slot].as_ref()?;
        Some(f(texture))
    }

    /// Returns information about the specified tile and its corresponding atlas, including the tile's bounds and the atlas's dimensions.
    fn get_texture_info(&self, id: &Key) -> Option<AtlasTextureInfo> {
        let tile = self.key_to_tile.get(id)?.clone();

        let storage = self.get_storage_read(&tile.texture.kind);

        let texture = storage[tile.texture.slot].as_ref()?;

        let info = AtlasTextureInfo {
            tile: tile.clone(),
            atlas_texture_size: texture.size,
        };

        Some(info)
    }

    fn create_texture(&mut self, size: Size<i32>, key: Key) -> AtlasTile {
        let kind = key.texture_kind();
        let storage = self.get_storage_write(&kind);

        let tile = {
            if let Some(tile) = storage
                .iter_mut()
                .flatten()
                .rev()
                .find_map(|texture| texture.allocate(size))
            {
                tile
            } else {
                let texture = self.push_texture(size, kind);
                texture.allocate(size).expect("Error allocating texture!")
            }
        };

        self.key_to_tile.insert(key, tile.clone());
        tile
    }

    /// Uploads data for the given tile
    pub fn upload_texture(&self, tile: &AtlasTile, data: &[u8]) {
        let storage = self.get_storage_read(&tile.texture.kind);
        let texture = storage[tile.texture.slot].as_ref();

        if let Some(texture) = texture {
            let tile_width: u32 = tile.bounds.size.width as _;
            let tile_height: u32 = tile.bounds.size.height as _;
            self.gpu.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &texture.raw,
                    aspect: wgpu::TextureAspect::All,
                    mip_level: 0,
                    origin: wgpu::Origin3d {
                        x: tile.bounds.origin.x as _,
                        y: tile.bounds.origin.y as _,
                        z: 0,
                    },
                },
                data,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(texture.kind.bytes_per_pixel() * tile_width),
                    rows_per_image: None,
                },
                wgpu::Extent3d {
                    width: tile_width,
                    height: tile_height,
                    depth_or_array_layers: 1,
                },
            );
        } else {
            log::error!("TEX_NOT_FOUND: Texture upload failed");
        }
    }

    fn push_texture(&mut self, size: Size<i32>, kind: TextureKind) -> &mut AtlasTexture {
        const DEFAULT_SIZE: Size<i32> = Size {
            width: 1024,
            height: 1024,
        };

        let size = DEFAULT_SIZE.max(&size);
        let format = kind.get_texture_format();

        let raw = self.gpu.create_texture(&wgpu::TextureDescriptor {
            label: Some("atlas_texture"),
            size: wgpu::Extent3d {
                width: size.width as u32,
                height: size.height as u32,
                depth_or_array_layers: 1,
            },
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            mip_level_count: 1,
            sample_count: 1,
            view_formats: &[],
        });

        let bytes_per_pixel = kind.bytes_per_pixel();

        let width = raw.width();
        let height = raw.height();

        let n_bytes = width * height * bytes_per_pixel;

        let init_data = vec![0u8; n_bytes as usize];

        self.gpu.queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &raw,
                aspect: wgpu::TextureAspect::All,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            &init_data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * bytes_per_pixel),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        let view = raw.create_view(&wgpu::TextureViewDescriptor::default());
        let allocator = etagere::BucketedAtlasAllocator::new(to_etagere_size(size));

        let storage = self.get_storage_write(&kind);
        let slot = storage.free_slots.pop();

        let atlas_tex = AtlasTexture {
            id: AtlasTextureId {
                kind,
                slot: slot.unwrap_or(storage.slots.len()),
            },
            allocator,
            kind,
            raw,
            view,
            format,
            size,
        };
        if let Some(slot) = slot {
            storage[slot] = Some(atlas_tex);
            storage.slots.get_mut(slot).unwrap().as_mut().unwrap()
        } else {
            storage.slots.push(Some(atlas_tex));
            storage.slots.last_mut().unwrap().as_mut().unwrap()
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct AtlasTextureId {
    pub kind: TextureKind,
    pub slot: usize,
}

impl std::fmt::Display for AtlasTextureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Atlas: kind = {} slot = {}", &self.kind, &self.slot)
    }
}

/// The big picture
pub struct AtlasTexture {
    // TODO add padding
    id: AtlasTextureId,
    raw: wgpu::Texture,
    allocator: etagere::BucketedAtlasAllocator,
    view: wgpu::TextureView,
    kind: TextureKind,
    format: TextureFormat,
    size: Size<i32>,
}

impl AtlasTexture {
    fn allocate(&mut self, size: Size<i32>) -> Option<AtlasTile> {
        let allocation = self.allocator.allocate(to_etagere_size(size))?;
        let id = allocation.id;

        let alloc_rect = allocation.rectangle;

        let bounds: Rect<i32> = Rect::from_origin_size(from_etagere_point(alloc_rect.min), size);

        Some(AtlasTile {
            id: id.into(),
            texture: self.id,
            bounds,
        })
    }

    pub fn id(&self) -> AtlasTextureId {
        self.id
    }

    pub fn kind(&self) -> TextureKind {
        self.kind
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn size(&self) -> Size<i32> {
        self.size
    }

    pub fn width(&self) -> i32 {
        self.size.width
    }

    pub fn height(&self) -> i32 {
        self.size.height
    }

    pub fn raw(&self) -> &GpuTexture {
        &self.raw
    }

    pub fn view(&self) -> &GpuTextureView {
        &self.view
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct AtlasTileId(u32);

#[derive(Debug, Clone, PartialEq)]
pub struct AtlasTile {
    /// This tile's id
    pub id: AtlasTileId,
    /// Which texture this tile belongs to ?
    pub texture: AtlasTextureId,
    /// Bounds of this tile
    pub bounds: Rect<i32>,
}

/// Contains information about the specified tile and its corresponding atlas, including the tile's bounds and the atlas's dimensions.
#[derive(Debug, Clone)]
pub struct AtlasTextureInfo {
    pub tile: AtlasTile,
    /// Size of the atlas in which the texture is in
    pub atlas_texture_size: Size<i32>,
}

impl AtlasTextureInfo {
    pub fn uv_to_atlas_space(&self, u: f32, v: f32) -> Vec2<f32> {
        // Scale the normalized coordinates (u, v) to the bounds of the texture tile in the atlas
        let tex_x = self.tile.bounds.origin.x as f32 + u * self.tile.bounds.size.width as f32;
        let tex_y = self.tile.bounds.origin.y as f32 + v * self.tile.bounds.size.height as f32;

        // Convert to atlas space
        let atlas_x = tex_x / self.atlas_texture_size.width as f32;
        let atlas_y = tex_y / self.atlas_texture_size.height as f32;

        Vec2::new(atlas_x, atlas_y)
    }
}

impl std::fmt::Debug for AtlasTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AtlasTexture")
            .field("id", &self.id)
            .field("kind", &self.kind)
            .field("format", &self.format)
            .field(
                "allocator",
                &format!("space = {}", self.allocator.allocated_space()),
            )
            .field("size", &self.size)
            .finish()
    }
}

impl From<etagere::AllocId> for AtlasTileId {
    fn from(value: etagere::AllocId) -> Self {
        Self(value.serialize())
    }
}

impl From<AtlasTileId> for etagere::AllocId {
    fn from(value: AtlasTileId) -> Self {
        etagere::AllocId::deserialize(value.0)
    }
}

#[derive(Debug, Default)]
struct AtlasTextureList<T: std::fmt::Debug> {
    slots: Vec<T>,
    free_slots: Vec<usize>,
}

impl<T: std::fmt::Debug> AtlasTextureList<T> {
    pub fn iter_mut(&mut self) -> std::slice::IterMut<'_, T> {
        self.slots.iter_mut()
    }
}

impl<T: std::fmt::Debug> std::ops::Index<usize> for AtlasTextureList<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.slots[index]
    }
}

impl<T: std::fmt::Debug> std::ops::IndexMut<usize> for AtlasTextureList<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.slots[index]
    }
}

fn to_etagere_size(size: Size<i32>) -> etagere::Size {
    etagere::size2(size.width, size.height)
}
fn from_etagere_point(p: etagere::Point) -> Vec2<i32> {
    Vec2 { x: p.x, y: p.y }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn should_convert_to_atlas_space() {
        let atlas_info = AtlasTextureInfo {
            tile: AtlasTile {
                id: AtlasTileId(0),
                texture: AtlasTextureId {
                    kind: TextureKind::Color,
                    slot: 0,
                },
                bounds: Rect::xywh(512, 512, 512, 512),
            },
            atlas_texture_size: Size {
                width: 1024,
                height: 1024,
            },
        };

        // Test a normalized coordinate at the center of the texture
        let tl = atlas_info.uv_to_atlas_space(0.0, 0.0);
        let tr = atlas_info.uv_to_atlas_space(1.0, 0.0);
        let bl = atlas_info.uv_to_atlas_space(0.0, 1.0);
        let br = atlas_info.uv_to_atlas_space(1.0, 1.0);

        assert_eq!(tl, (0.5, 0.5).into());
        assert_eq!(tr, (1.0, 0.5).into());
        assert_eq!(bl, (0.5, 1.0).into());
        assert_eq!(br, (1.0, 1.0).into());
    }

    #[test]
    fn should_convert_to_atlas_space_with_small_texture() {
        let atlas_info = AtlasTextureInfo {
            tile: AtlasTile {
                id: AtlasTileId(0),
                texture: AtlasTextureId {
                    kind: TextureKind::Color,
                    slot: 0,
                },
                bounds: Rect::xywh(0, 0, 128, 128),
            },
            atlas_texture_size: Size {
                width: 1024,
                height: 1024,
            },
        };

        // Test normalized coordinates at the center of the texture
        let center = atlas_info.uv_to_atlas_space(0.5, 0.5);

        assert_eq!(center, Vec2::new(0.0625, 0.0625)); // (128 / 1024)

        // Test corner cases
        let top_left = atlas_info.uv_to_atlas_space(0.0, 0.0);
        let top_right = atlas_info.uv_to_atlas_space(1.0, 0.0);
        let bottom_left = atlas_info.uv_to_atlas_space(0.0, 1.0);
        let bottom_right = atlas_info.uv_to_atlas_space(1.0, 1.0);

        assert_eq!(top_left, Vec2::new(0.0, 0.0));
        assert_eq!(top_right, Vec2::new(0.125, 0.0)); // (128 / 1024)
        assert_eq!(bottom_left, Vec2::new(0.0, 0.125)); // (128 / 1024)
        assert_eq!(bottom_right, Vec2::new(0.125, 0.125)); // (128 / 1024)
    }

    #[test]
    fn should_convert_to_atlas_space_1x1_texture() {
        let atlas_info = AtlasTextureInfo {
            tile: AtlasTile {
                id: AtlasTileId(0),
                texture: AtlasTextureId {
                    kind: TextureKind::Color,
                    slot: 0,
                },
                bounds: Rect::xywh(800, 800, 1, 1),
            },
            atlas_texture_size: Size {
                width: 1024,
                height: 1024,
            },
        };

        let tl = atlas_info.uv_to_atlas_space(0.0, 0.0);
        let br = atlas_info.uv_to_atlas_space(1.0, 1.0);

        assert_eq!(tl, Vec2::new(800.0 / 1024.0, 800.0 / 1024.0)); // Atlas X position (800) mapped into the atlas space (1024).
        assert_eq!(
            br,
            Vec2::new((800.0 + 1.0) / 1024.0, (800.0 + 1.0) / 1024.0)
        ); // Atlas X position (800) mapped into the atlas space (1024).
    }
}
