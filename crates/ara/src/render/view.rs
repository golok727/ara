mod system;
pub use system::*;

use ara_math::Size;

#[derive(Debug, Clone, Copy)]
pub struct ViewConfig {
    pub size: Size<u32>,
    pub resolution: f32,
    pub antialias: bool,
    pub texture_format: wgpu::TextureFormat,
    pub usage: wgpu::TextureUsages,
}

impl Default for ViewConfig {
    fn default() -> Self {
        Self {
            size: Size::new(800, 600),
            resolution: 1.0,
            antialias: true,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            texture_format: wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

#[derive(Default)]
pub enum ViewTarget {
    Surface(wgpu::SurfaceTarget<'static>),
    // todo
    // Image(ImageHandle),
    #[default]
    Empty,
}

impl<T> From<T> for ViewTarget
where
    T: Into<wgpu::SurfaceTarget<'static>>,
{
    fn from(target: T) -> Self {
        Self::Surface(target.into())
    }
}
