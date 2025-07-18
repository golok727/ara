use derive_more::derive::Deref;
use wgpu::SurfaceTexture;

use crate::{
    gpu,
    render::{texture::TextureSource, Item},
};

use super::RenderTargetAdapter;

#[derive(Deref, Debug, Clone, Eq, PartialEq, Hash)]
pub struct BackendRenderTargetHandle(pub(crate) Item<BackendRenderTarget>);

impl From<BackendRenderTargetHandle> for super::RenderTarget {
    fn from(handle: BackendRenderTargetHandle) -> Self {
        Self::Backend(handle)
    }
}

pub struct BackendRenderTarget {
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl BackendRenderTarget {
    pub fn new(
        context: &gpu::Context,
        into_surface: impl Into<gpu::SurfaceTarget<'static>>,
        texture_source: &TextureSource<()>,
    ) -> Self {
        let surface = context
            .instance
            .create_surface(into_surface)
            .expect("Error creating surface");

        let capabilities = surface.get_capabilities(&context.adapter);

        let size = texture_source.pixel_size();

        let surface_config: wgpu::wgt::SurfaceConfiguration<Vec<wgpu::TextureFormat>> =
            wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | texture_source.usage,
                format: texture_source.format,
                width: size.width,
                height: size.height,
                present_mode: capabilities.present_modes[0],
                alpha_mode: capabilities.alpha_modes[0],
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

        surface.configure(context, &surface_config);

        BackendRenderTarget {
            surface,
            config: surface_config,
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.config.width != width || self.config.height != height {
            self.config.width = width;
            self.config.height = height;
            self.surface.configure(device, &self.config);
        }
    }
}

#[derive(Default)]
pub struct BackendRenderTargetAdapter {
    current_texture: Option<SurfaceTexture>,
}

impl RenderTargetAdapter for BackendRenderTargetAdapter {
    type Target = BackendRenderTarget;

    fn begin_pass<'encoder>(
        &mut self,
        target: &mut Self::Target,
        clear_color: crate::Color,
        encoder: &'encoder mut wgpu::CommandEncoder,
        _cx: &mut crate::render::RenderContext,
    ) -> Option<wgpu::RenderPass<'encoder>> {
        let current_texture = target.surface.get_current_texture().ok()?;

        let view = current_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let pass = encoder.begin_render_pass(
            &(wgpu::RenderPassDescriptor {
                label: Some("ara_render::backend_target::RenderPass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(clear_color.into()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            }),
        );

        self.current_texture = Some(current_texture);

        Some(pass)
    }

    fn render_complete(&mut self) {
        if let Some(current_texture) = self.current_texture.take() {
            current_texture.present();
        }
    }
}
