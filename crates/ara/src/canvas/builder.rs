use std::sync::Arc;

use wgpu::{ TextureFormat, TextureUsages };

use crate::{ renderer::create_ara_renderer, GpuContext, Renderer2DSpecs, AraAtlas, TextSystem };

use super::{ surface::CanvasSurfaceConfig, Canvas };

#[derive(Default)]
pub struct CanvasBuilder {
    pub(super) texture_atlas: Option<Arc<AraAtlas>>,
    pub(super) text_system: Option<Arc<TextSystem>>,
    pub(super) surface_config: CanvasSurfaceConfig,
}

impl CanvasBuilder {
    pub fn msaa_samples(mut self, sample_count: u32) -> Self {
        self.surface_config.msaa_sample_count = sample_count;
        self
    }

    pub fn width(mut self, width: u32) -> Self {
        self.surface_config.width = width.max(1);
        self
    }

    pub fn height(mut self, height: u32) -> Self {
        self.surface_config.height = height.max(1);
        self
    }

    pub fn add_surface_usage(mut self, usage: TextureUsages) -> Self {
        self.surface_config.usage |= usage;
        self
    }

    pub fn surface_format(mut self, format: TextureFormat) -> Self {
        self.surface_config.format = format;
        self
    }

    pub fn build(self, gpu: GpuContext) -> Canvas {
        log::info!("Creating canvas with surface_config: {:#?}", self.surface_config);

        let texture_atlas = self.texture_atlas.unwrap_or(Arc::new(AraAtlas::new(gpu.clone())));

        let text_system = self.text_system.unwrap_or(Arc::new(TextSystem::default()));

        let renderer = create_ara_renderer(
            gpu,
            &texture_atlas,
            &(Renderer2DSpecs {
                width: self.surface_config.width,
                height: self.surface_config.height,
                msaa_sample_count: self.surface_config.msaa_sample_count,
            })
        );

        Canvas::new(self.surface_config, renderer, texture_atlas, text_system)
    }

    pub fn with_texture_atlas(mut self, atlas: Arc<AraAtlas>) -> Self {
        self.texture_atlas = Some(atlas);
        self
    }

    pub fn with_text_system(mut self, text_system: Arc<TextSystem>) -> Self {
        self.text_system = Some(text_system);
        self
    }
}
