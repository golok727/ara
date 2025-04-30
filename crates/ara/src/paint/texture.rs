use crate::math::Size;

use super::{atlas::AtlasTextureId, AtlasImage, AtlasKey};

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TextureId {
    /// Internal things created by ara
    Internal(usize),
    // Any texture created by user
    User(usize),

    AtlasKey(AtlasKey),

    Atlas(AtlasTextureId),
}

impl From<AtlasImage> for TextureId {
    fn from(image: AtlasImage) -> Self {
        Self::AtlasKey(image.into())
    }
}

impl From<AtlasKey> for TextureId {
    fn from(atlas: AtlasKey) -> Self {
        Self::AtlasKey(atlas)
    }
}

impl From<AtlasTextureId> for TextureId {
    fn from(atlas: AtlasTextureId) -> Self {
        Self::Atlas(atlas)
    }
}

impl Default for TextureId {
    fn default() -> Self {
        Self::WHITE_TEXTURE
    }
}

impl TextureId {
    pub const WHITE_TEXTURE: Self = TextureId::AtlasKey(AtlasKey::WhiteTexture);
    #[inline(always)]
    pub fn is_white(&self) -> bool {
        self == &Self::WHITE_TEXTURE
    }
}

pub static WHITE_UV: (f32, f32) = (0.0, 0.0);

pub struct Texture2DSpecs {
    pub size: Size<u32>,
    pub format: TextureFormat,
}

pub type TextureAddressMode = wgpu::AddressMode;
pub type TextureFilterMode = wgpu::FilterMode;

#[derive(Debug, Clone, Default)]
pub struct TextureOptions {
    pub address_mode_u: TextureAddressMode,
    pub address_mode_v: TextureAddressMode,
    pub address_mode_w: TextureAddressMode,
    pub mag_filter: TextureFilterMode,
    pub min_filter: TextureFilterMode,
    pub mipmap_filter: TextureFilterMode,
    pub kind: TextureKind,
}

impl TextureOptions {
    pub fn mag_filter(mut self, mode: TextureFilterMode) -> Self {
        self.mag_filter = mode;
        self
    }

    pub fn kind(mut self, kind: TextureKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn min_filter(mut self, mode: TextureFilterMode) -> Self {
        self.min_filter = mode;
        self
    }

    pub fn mip_map_filter(mut self, mode: TextureFilterMode) -> Self {
        self.min_filter = mode;
        self
    }

    pub fn address_mode_u(mut self, mode: TextureAddressMode) -> Self {
        self.address_mode_u = mode;
        self
    }

    pub fn address_mode_v(mut self, mode: TextureAddressMode) -> Self {
        self.address_mode_v = mode;
        self
    }

    pub fn address_mode_w(mut self, mode: TextureAddressMode) -> Self {
        self.address_mode_w = mode;
        self
    }
}

pub struct Texture2D {}

impl std::fmt::Display for TextureId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Internal(id) => write!(f, "Texture::Internal({})", id),
            Self::User(id) => write!(f, "Texture::User({})", id),
            Self::Atlas(id) => write!(f, "Texture::AtlasTexture({})", id),
            Self::AtlasKey(id) => write!(f, "Texture::AtlasTexture({:#?})", id),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Hash, Eq)]
#[repr(usize)]
pub enum TextureKind {
    // single channel;
    Mask = 0,
    #[default]
    Color = 1,
}

impl TextureKind {
    pub fn get_texture_format(&self) -> TextureFormat {
        match self {
            Self::Mask => TextureFormat::R8Unorm,
            // FIXME: should we use Bgara ?
            Self::Color => TextureFormat::Rgba8Unorm,
        }
    }

    pub fn bytes_per_pixel(&self) -> u32 {
        match self {
            TextureKind::Color => 4,
            TextureKind::Mask => 1,
        }
    }

    pub fn is_color(&self) -> bool {
        matches!(self, Self::Color)
    }

    pub fn is_mask(&self) -> bool {
        matches!(self, Self::Color)
    }
}

impl std::fmt::Display for TextureKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Color => write!(f, "Kind::Color"),
            Self::Mask => write!(f, "Kind::Gray"),
        }
    }
}

// re-export
pub use wgpu::{
    Texture as GpuTexture, TextureView as GpuTextureView,
    TextureViewDescriptor as GpuTextureViewDescriptor,
};
pub type TextureFormat = wgpu::TextureFormat;
