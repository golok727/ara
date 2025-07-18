use ara_math::Size;

use crate::render::ItemManager;

const MIN_SIZE: Size<u32> = Size {
    width: 1,
    height: 1,
};

#[derive(Debug)]
pub struct TextureSource<T: 'static = ()> {
    pub(crate) source: T,
    // the logical size of the texture
    pub(crate) size: Size<u32>,

    pub(crate) antialias: bool,
    // the physical size of the texture, ie. size * resolution
    pub(crate) pixel_size: Size<u32>,

    pub(crate) resolution: f32,

    pub(crate) usage: wgpu::TextureUsages,

    pub(crate) format: wgpu::TextureFormat,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TextureSourceDescriptor {
    pub size: Size<u32>,
    pub resolution: f32,
    pub antialias: bool,
    pub usage: wgpu::TextureUsages,
    pub format: wgpu::TextureFormat,
}

impl Default for TextureSourceDescriptor {
    fn default() -> Self {
        Self {
            size: Size::new(800, 600),
            resolution: 1.0,
            antialias: true,
            usage: wgpu::TextureUsages::empty(),
            format: wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

impl TextureSource<()> {
    pub fn empty(options: &TextureSourceDescriptor) -> TextureSource<()> {
        let size = options.size.max(&MIN_SIZE);
        let pixel_size = size.map(|s| ((s as f32) * options.resolution).round() as u32);

        TextureSource {
            source: (),
            size,
            pixel_size,
            resolution: options.resolution,
            antialias: options.antialias,
            usage: options.usage,
            format: options.format,
        }
    }
}

impl<T> TextureSource<T>
where
    T: 'static,
{
    pub fn source(&self) -> &T {
        &self.source
    }

    pub fn replace<U>(self, resource: U) -> TextureSource<U> {
        TextureSource {
            source: resource,
            size: self.size,
            pixel_size: self.pixel_size,
            resolution: self.resolution,
            antialias: self.antialias,
            usage: self.usage,
            format: self.format,
        }
    }

    pub fn new(resource: T, options: &TextureSourceDescriptor) -> Self {
        TextureSource::empty(options).replace(resource)
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }
    pub fn usage(&self) -> wgpu::TextureUsages {
        self.usage
    }

    pub fn antialias(&self) -> bool {
        self.antialias
    }

    pub fn width(&self) -> u32 {
        self.size.width
    }

    pub fn height(&self) -> u32 {
        self.size.height
    }

    pub fn size(&self) -> Size<u32> {
        self.size
    }

    pub fn pixel_width(&self) -> u32 {
        self.pixel_size.width
    }

    pub fn pixel_height(&self) -> u32 {
        self.pixel_size.height
    }

    pub fn pixel_size(&self) -> Size<u32> {
        self.pixel_size
    }

    pub fn resolution(&self) -> f32 {
        self.resolution
    }

    #[inline]
    pub(super) fn resize_impl(&mut self, logical_size: Size<u32>, resolution: f32) -> bool {
        assert!(resolution > 0.0, "Resolution must be greater than 0");
        self.size = logical_size.max(&MIN_SIZE);
        self.resolution = resolution;
        let old_pixel_size = self.pixel_size;
        let new_pixel_size = self
            .size
            .map(|s| ((s as f32) * self.resolution).round() as u32);
        self.pixel_size = new_pixel_size;

        old_pixel_size != new_pixel_size
    }
}

impl<T> TextureSource<T>
where
    T: RenderTexture,
{
    pub fn resize(&mut self, cx: &mut impl ItemManager, logical_size: Size<u32>) {
        if self.resize_impl(logical_size, self.resolution) {
            self.source.resize(cx, self.pixel_size);
        }
    }

    pub fn set_resolution(&mut self, cx: &mut impl ItemManager, resolution: f32) {
        if self.resize_impl(self.size, resolution) {
            self.source.resize(cx, self.pixel_size);
        }
    }
}

pub trait RenderTexture {
    fn resize(&self, cx: &mut impl ItemManager, physical_size: Size<u32>);
}
