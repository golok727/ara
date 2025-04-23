use crate::{ canvas::surface::create_mssa_view, GpuContext };

use super::{
    snapshot::CanvasSnapshotSource,
    surface::{ CanvasSurface, CanvasSurfaceConfig },
    Canvas,
};
use anyhow::Result;

pub struct OffscreenRenderTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    msaa_sample_count: u32,
    mssa_view: Option<wgpu::TextureView>,
}

impl OffscreenRenderTarget {
    pub(super) fn new(gpu: &GpuContext, config: &CanvasSurfaceConfig) -> Self {
        let texture = create_fb_texture(gpu, config);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            msaa_sample_count: config.msaa_sample_count,
            mssa_view: create_mssa_view(gpu, config),
        }
    }
}

impl CanvasSurface for OffscreenRenderTarget {
    type PaintOutput = ();
    const LABEL: &'static str = "OffscreenRenderTarget";

    fn configure(&mut self, gpu: &GpuContext, config: &CanvasSurfaceConfig) {
        debug_assert!(config.width != 0, "Got zero width");
        debug_assert!(config.height != 0, "Got zero height");

        let texture = create_fb_texture(gpu, config);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.mssa_view = create_mssa_view(gpu, config);
        self.texture = texture;
        self.view = view;
    }

    fn get_config(&self) -> CanvasSurfaceConfig {
        CanvasSurfaceConfig {
            width: self.texture.width(),
            height: self.texture.height(),
            format: self.texture.format(),
            usage: self.texture.usage(),
            msaa_sample_count: self.msaa_sample_count,
        }
    }

    fn paint(&mut self, canvas: &mut Canvas) -> Result<Self::PaintOutput> {
        let (view, resolve_target) = (self.msaa_sample_count > 1)
            .then_some(self.mssa_view.as_ref())
            .flatten()
            .map_or((&self.view, None), |texture_view| { (texture_view, Some(&self.view)) });

        canvas.render_to_texture(view, resolve_target);
        Ok(())
    }
}

impl Canvas {
    pub fn create_offscreen_target(&self) -> OffscreenRenderTarget {
        OffscreenRenderTarget::new(self.renderer.gpu(), &self.surface_config)
    }
}

impl CanvasSnapshotSource for OffscreenRenderTarget {
    fn get_source_texture(&self) -> wgpu::Texture {
        self.texture.clone()
    }
}

fn create_fb_texture(gpu: &GpuContext, config: &CanvasSurfaceConfig) -> wgpu::Texture {
    gpu.create_texture(
        &(wgpu::TextureDescriptor {
            label: Some("framebuffer"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | config.usage,
            view_formats: &[config.format],
        })
    )
}
