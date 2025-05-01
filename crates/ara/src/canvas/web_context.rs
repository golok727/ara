use std::ops::Deref;

use crate::gpu::Context;
use crate::gpu::ContextSpecification;
use crate::{ BackendRenderingContext, RenderContext, RenderContextConfig };

use super::CanvasConfig;

#[cfg(target_arch = "wasm32")]
#[derive(Clone)]
pub enum WebSurfaceTarget {
    OffscreenCanvas(web_sys::OffscreenCanvas),
    Canvas(web_sys::HtmlCanvasElement),
}

#[cfg(target_arch = "wasm32")]
impl WebSurfaceTarget {
    pub fn try_as_canvas(&self) -> Option<&web_sys::HtmlCanvasElement> {
        match self {
            WebSurfaceTarget::OffscreenCanvas(_) => None,
            WebSurfaceTarget::Canvas(c) => Some(c),
        }
    }

    pub fn try_as_offscreen_canvas(&self) -> Option<&web_sys::OffscreenCanvas> {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => Some(c),
            WebSurfaceTarget::Canvas(_) => None,
        }
    }

    pub fn canvas(&self) -> &web_sys::HtmlCanvasElement {
        self.try_as_canvas().expect("Expected a canvas")
    }

    pub fn offscreen_canvas(&self) -> &web_sys::OffscreenCanvas {
        self.try_as_offscreen_canvas().expect("Expected an offscreen canvas")
    }

    pub fn width(&self) -> u32 {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => c.width(),
            WebSurfaceTarget::Canvas(c) => c.width(),
        }
    }
    pub fn height(&self) -> u32 {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => c.height(),
            WebSurfaceTarget::Canvas(c) => c.height(),
        }
    }
    pub fn set_width(&self, width: u32) {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => c.set_width(width),
            WebSurfaceTarget::Canvas(c) => c.set_width(width),
        }
    }
    pub fn set_height(&self, height: u32) {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => c.set_height(height),
            WebSurfaceTarget::Canvas(c) => c.set_height(height),
        }
    }

    fn create_gpu_surface(
        &self,
        instance: &wgpu::Instance
    ) -> anyhow::Result<wgpu::Surface<'static>> {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => {
                let surface_target = wgpu::SurfaceTarget::OffscreenCanvas(c.clone());
                let surface = instance.create_surface(surface_target)?;
                Ok(surface)
            }
            WebSurfaceTarget::Canvas(c) => {
                let surface_target = wgpu::SurfaceTarget::Canvas(c.clone());
                let surface = instance.create_surface(surface_target)?;
                Ok(surface)
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub struct WebRenderingContext {
    inner: BackendRenderingContext<'static>,
    surface_target: WebSurfaceTarget,
    gpu: Context,
}

impl Deref for WebRenderingContext {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.gpu
    }
}

#[cfg(target_arch = "wasm32")]
impl WebRenderingContext {
    pub fn gpu(&self) -> &Context {
        &self.gpu
    }

    pub fn surface_target(&self) -> &WebSurfaceTarget {
        &self.surface_target
    }

    pub(crate) async fn create(
        surface_target: impl Into<WebSurfaceTarget>,
        render_target_config: &RenderContextConfig
    ) -> anyhow::Result<Self> {
        let width = render_target_config.width;
        let height = render_target_config.height;

        let web_surface: WebSurfaceTarget = surface_target.into();
        web_surface.set_width(width);
        web_surface.set_height(height);

        let instance = crate::gpu::create_instance_with_wgpu_detection(
            &wgpu::InstanceDescriptor::default()
        ).await;

        let surface = web_surface.create_gpu_surface(&instance)?;

        let gpu = Context::create(
            instance,
            &(ContextSpecification {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
        ).await?;

        let target = BackendRenderingContext::new(&gpu, surface, &render_target_config)?;

        let this = Self {
            inner: target,
            gpu,
            surface_target: web_surface,
        };

        Ok(this)
    }
}

#[cfg(target_arch = "wasm32")]
impl RenderContext for WebRenderingContext {
    type PaintOutput = ();
    const LABEL: &'static str = "WebRenderContext";

    fn paint(&mut self, canvas: &mut super::Canvas) -> anyhow::Result<Self::PaintOutput> {
        self.inner.paint(canvas)?;
        Ok(())
    }

    fn configure(&mut self, gpu: &Context, config: &super::RenderContextConfig) {
        self.inner.configure(gpu, config);
    }

    fn get_config(&self) -> super::RenderContextConfig {
        self.inner.get_config()
    }
}

#[cfg(target_arch = "wasm32")]
impl From<web_sys::HtmlCanvasElement> for WebSurfaceTarget {
    fn from(canvas: web_sys::HtmlCanvasElement) -> Self {
        Self::Canvas(canvas)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<web_sys::OffscreenCanvas> for WebSurfaceTarget {
    fn from(canvas: web_sys::OffscreenCanvas) -> Self {
        Self::OffscreenCanvas(canvas)
    }
}

#[cfg(target_arch = "wasm32")]
impl super::Canvas {
    pub fn new_web(target: &WebRenderingContext, config: CanvasConfig) -> Self {
        Self::new(target.gpu.clone(), config)
    }
}
