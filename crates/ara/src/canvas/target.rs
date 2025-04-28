use crate::{ gpu, GpuContext };
use anyhow::Result;

use super::Canvas;

pub trait RenderTarget {
    type PaintOutput;
    const LABEL: &'static str;

    fn paint(&mut self, canvas: &mut Canvas) -> Result<Self::PaintOutput>;
    fn configure(&mut self, gpu: &GpuContext, config: &RenderTargetConfig);
    fn get_config(&self) -> RenderTargetConfig;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderTargetConfig {
    pub width: u32,
    pub height: u32,
    pub format: gpu::TextureFormat,
    pub usage: gpu::TextureUsages,
    pub(crate) msaa_sample_count: u32,
}

impl Default for RenderTargetConfig {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            format: gpu::TextureFormat::Rgba8Unorm,
            usage: gpu::TextureUsages::RENDER_ATTACHMENT,
            msaa_sample_count: 1,
        }
    }
}

pub fn create_msaa_view(
    device: &wgpu::Device,
    config: &RenderTargetConfig
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
