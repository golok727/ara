#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::*;

mod system;
pub use system::RenderTargetSystem;

mod backend;
pub use backend::{BackendRenderTarget, BackendRenderTargetHandle};

use crate::gpu::{self};

use super::{texture::RenderTexture, ItemManager};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTargetConfig {
    pub width: u32,
    pub height: u32,
    pub format: gpu::TextureFormat,
    pub usage: gpu::TextureUsages,
    /// enables antialiasing for this render target
    pub antialias: bool,
}

impl RenderTargetConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn width(mut self, width: u32) -> Self {
        self.width = width.max(1);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.height = height.max(1);
        self
    }

    pub fn add_surface_usage(mut self, usage: gpu::TextureUsages) -> Self {
        self.usage |= usage;
        self
    }

    pub fn surface_format(mut self, format: gpu::TextureFormat) -> Self {
        self.format = format;
        self
    }

    pub fn antialias(mut self, antialias: bool) -> Self {
        self.antialias = antialias;
        self
    }
}

impl Default for RenderTargetConfig {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            format: gpu::TextureFormat::Rgba8Unorm,
            usage: gpu::TextureUsages::RENDER_ATTACHMENT,
            antialias: false,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum RenderTarget {
    Backend(BackendRenderTargetHandle),
    Noop,
}

impl RenderTexture for RenderTarget {
    fn resize(&self, cx: &mut impl ItemManager, physical_size: ara_math::Size<u32>) {
        match self {
            RenderTarget::Backend(handle) => {
                let _ = handle.update(cx, |target, cx| {
                    target.resize(&cx.gpu.device, physical_size.width, physical_size.height);
                });
            }
            RenderTarget::Noop => {
                // No operation for noop targets
            }
        }
    }
}

impl RenderTarget {
    pub fn backend(handle: BackendRenderTargetHandle) -> Self {
        Self::Backend(handle)
    }

    pub fn noop() -> Self {
        Self::Noop
    }
}

pub trait RenderTargetAdapter {
    type Target;
    fn begin_pass<'encoder>(
        &mut self,
        target: &mut Self::Target,
        clear_color: crate::Color,
        encoder: &'encoder mut wgpu::CommandEncoder,
        cx: &mut crate::render::RenderContext,
    ) -> Option<wgpu::RenderPass<'encoder>>;

    fn render_complete(&mut self);
}
