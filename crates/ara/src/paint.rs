pub mod atlas;
pub mod brush;
pub mod color;
pub mod draw_list;
pub mod geometry;
pub mod graphics_instruction;
pub mod image;
pub mod mesh;
pub mod primitives;
pub mod stroke_tessellate;
pub mod text;
pub mod texture;

use crate::{ math::Vec2, text::GlyphImage };

pub use atlas::*;
pub use brush::*;
pub use color::*;
pub use draw_list::*;
pub use geometry::*;
pub use graphics_instruction::*;
pub use image::*;
pub use mesh::*;
pub use primitives::*;
pub use stroke_tessellate::*;
pub use text::*;
pub use texture::*;

pub type AraAtlasTextureInfoMap = AtlasTextureInfoMap<AtlasKey>;
pub const DEFAULT_UV_COORD: Vec2<f32> = Vec2 { x: 0.0, y: 0.0 };

pub type AraAtlas = TextureAtlas<AtlasKey>;
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub enum AtlasKey {
    Image(AtlasImage),
    Glyf(GlyphImage),
    WhiteTexture,
}

impl AtlasKeySource for AtlasKey {
    fn texture_kind(&self) -> TextureKind {
        match self {
            AtlasKey::Glyf(glyph) => {
                if glyph.is_emoji { TextureKind::Color } else { TextureKind::Mask }
            }
            AtlasKey::Image(image) => image.texture_kind,
            AtlasKey::WhiteTexture => TextureKind::Color,
        }
    }
}

impl From<GlyphImage> for AtlasKey {
    fn from(atlas_glyf: GlyphImage) -> Self {
        Self::Glyf(atlas_glyf)
    }
}

impl From<AtlasImage> for AtlasKey {
    fn from(image: AtlasImage) -> Self {
        Self::Image(image)
    }
}
