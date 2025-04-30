use crate::{ gpu, Context };
use anyhow::Result;

use super::Canvas;

pub trait CanvasRenderingContext {
    type PaintOutput;
    const LABEL: &'static str;

    fn paint(&mut self, canvas: &mut Canvas) -> Result<Self::PaintOutput>;
    fn configure(&mut self, gpu: &Context, config: &CanvasRenderingContextConfig);
    fn get_config(&self) -> CanvasRenderingContextConfig;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MsaaSampleLevel {
    None = 1,
    Four = 4,
    Eight = 8,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanvasRenderingContextConfig {
    pub width: u32,
    pub height: u32,
    pub format: gpu::TextureFormat,
    pub usage: gpu::TextureUsages,
    pub(crate) msaa_sample_count: u32,
}

impl CanvasRenderingContextConfig {
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

    pub fn msaa_samples(mut self, level: MsaaSampleLevel) -> Self {
        self.msaa_sample_count = level as u32;
        self
    }
}

impl Default for CanvasRenderingContextConfig {
    fn default() -> Self {
        Self {
            width: 1,
            height: 1,
            format: gpu::TextureFormat::Rgba8Unorm,
            usage: gpu::TextureUsages::RENDER_ATTACHMENT,
            msaa_sample_count: 1,
        }
    }
}

pub fn create_msaa_view(
    device: &wgpu::Device,
    config: &CanvasRenderingContextConfig
) -> Option<wgpu::TextureView> {
    (config.msaa_sample_count > 1).then(|| {
        let texture_format = config.format;

        device
            .create_texture(
                &(wgpu::TextureDescriptor {
                    label: Some("ara_msaa_texture"),
                    size: wgpu::Extent3d {
                        width: config.width,
                        height: config.height,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count: config.msaa_sample_count.max(1),
                    dimension: wgpu::TextureDimension::D2,
                    format: texture_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[texture_format],
                })
            )
            .create_view(&wgpu::TextureViewDescriptor::default())
    })
}
