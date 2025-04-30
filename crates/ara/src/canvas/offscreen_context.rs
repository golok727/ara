use crate::{canvas::render_context::create_msaa_view, Context};

use super::{
    render_context::{RenderContext, RenderContextConfig},
    snapshot::CanvasSnapshotSource,
    Canvas,
};
use anyhow::Result;

pub struct OffscreenRenderContext {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    msaa_sample_count: u32,
    msaa_view: Option<wgpu::TextureView>,
}

impl OffscreenRenderContext {
    pub(super) fn new(gpu: &Context, config: &RenderContextConfig) -> Self {
        let texture = create_framebuffer_texture(gpu, config);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            msaa_sample_count: config.msaa_sample_count,
            msaa_view: create_msaa_view(&gpu.device, config),
        }
    }
}

impl RenderContext for OffscreenRenderContext {
    type PaintOutput = ();
    const LABEL: &'static str = "OffscreenRenderContext";

    fn configure(&mut self, gpu: &Context, config: &RenderContextConfig) {
        debug_assert!(config.width != 0, "Got zero width");
        debug_assert!(config.height != 0, "Got zero height");

        let texture = create_framebuffer_texture(gpu, config);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.msaa_view = create_msaa_view(gpu, config);
        self.texture = texture;
        self.view = view;
    }

    fn get_config(&self) -> RenderContextConfig {
        RenderContextConfig {
            width: self.texture.width(),
            height: self.texture.height(),
            format: self.texture.format(),
            usage: self.texture.usage(),
            msaa_sample_count: self.msaa_sample_count,
        }
    }

    fn paint(&mut self, canvas: &mut Canvas) -> Result<Self::PaintOutput> {
        let (view, resolve_target) = (self.msaa_sample_count > 1)
            .then_some(self.msaa_view.as_ref())
            .flatten()
            .map_or((&self.view, None), |texture_view| {
                (texture_view, Some(&self.view))
            });

        canvas.render_to_texture(view, resolve_target);
        Ok(())
    }
}

impl Canvas {
    pub fn create_offscreen_target(&self) -> OffscreenRenderContext {
        OffscreenRenderContext::new(self.renderer.gpu(), &self.context_cfg)
    }
}

impl CanvasSnapshotSource for OffscreenRenderContext {
    fn get_source_texture(&self) -> wgpu::Texture {
        self.texture.clone()
    }
}

fn create_framebuffer_texture(gpu: &Context, config: &RenderContextConfig) -> wgpu::Texture {
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
        }),
    )
}
