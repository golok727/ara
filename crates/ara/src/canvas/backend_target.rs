use std::ops::Deref;

use crate::canvas::surface::CanvasSurface;
use crate::{Canvas, GpuContext};
use anyhow::Result;
use wgpu::SurfaceTexture;

use super::surface::{create_mssa_view, CanvasSurfaceConfig};

#[derive(Debug, Clone)]
pub struct GpuSurfaceSpecification {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct BackendRenderTarget<'a> {
    surface: wgpu::Surface<'a>,
    config: wgpu::SurfaceConfiguration,
    msaa_sample_count: u32,
    msaa_view: Option<wgpu::TextureView>,
}

impl<'a> Deref for BackendRenderTarget<'a> {
    type Target = wgpu::Surface<'a>;

    fn deref(&self) -> &Self::Target {
        &self.surface
    }
}

impl<'a> BackendRenderTarget<'a> {
    fn new(
        gpu: &GpuContext,
        surface_target: impl Into<wgpu::SurfaceTarget<'a>>,
        config: &CanvasSurfaceConfig,
    ) -> Result<Self> {
        let surface = gpu.instance.create_surface(surface_target)?;

        let capabilities = surface.get_capabilities(&gpu.adapter);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | config.usage,
            format: config.format,
            width: config.width,
            height: config.height,
            present_mode: capabilities.present_modes[0],
            alpha_mode: capabilities.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&gpu.device, &surface_config);

        Ok(Self {
            surface,
            config: surface_config,
            msaa_sample_count: config.msaa_sample_count,
            msaa_view: create_mssa_view(gpu, config),
        })
    }
}

#[derive(Debug)]
pub struct PaintedSurface(SurfaceTexture);

impl PaintedSurface {
    pub fn present(self) {
        self.0.present()
    }
}

impl<'a> CanvasSurface for BackendRenderTarget<'a> {
    type PaintOutput = PaintedSurface;
    const LABEL: &'static str = "BackendRenderTarget";

    fn paint(&mut self, canvas: &mut Canvas) -> Result<Self::PaintOutput> {
        let surface_texture = self.surface.get_current_texture()?;

        let view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let (view, resolve_target) = (self.msaa_sample_count > 1)
            .then_some(self.msaa_view.as_ref())
            .flatten()
            .map_or((&view, None), |texture_view| (texture_view, Some(&view)));

        canvas.render_to_texture(view, resolve_target);

        Ok(PaintedSurface(surface_texture))
    }

    fn configure(&mut self, gpu: &GpuContext, config: &CanvasSurfaceConfig) {
        self.config.width = config.width;
        self.config.height = config.height;
        self.config.usage = config.usage | wgpu::TextureUsages::RENDER_ATTACHMENT;
        self.config.format = config.format;

        self.msaa_view = create_mssa_view(gpu, config);
        self.surface.configure(&gpu.device, &self.config);
    }

    fn get_config(&self) -> CanvasSurfaceConfig {
        CanvasSurfaceConfig {
            width: self.config.width,
            height: self.config.height,
            format: self.config.format,
            usage: self.config.usage,
            msaa_sample_count: self.msaa_sample_count,
        }
    }
}

impl Canvas {
    pub fn create_backend_target<'window>(
        &self,
        surface_target: impl Into<wgpu::SurfaceTarget<'window>>,
    ) -> Result<BackendRenderTarget<'window>> {
        BackendRenderTarget::new(self.renderer.gpu(), surface_target, &self.surface_config)
    }
}
