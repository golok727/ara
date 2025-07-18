use ara_math::Size;
use derive_more::derive::{Deref, DerefMut};

use crate::render::{
    render_target::{BackendRenderTarget, BackendRenderTargetHandle, RenderTarget},
    systems::System,
    texture::{TextureSource, TextureSourceDescriptor},
    ItemContext, ItemManager, RenderContext, WithRenderContext,
};

use super::{ViewConfig, ViewTarget};

#[derive(Deref, DerefMut)]
pub struct ViewSource(TextureSource<RenderTarget>);

pub struct ViewSystem {
    view: ViewSource,
}

impl System for ViewSystem {
    fn init(&mut self, _cx: &mut RenderContext)
    where
        Self: Sized,
    {
    }
}

impl ViewSystem {
    fn create_view(
        cx: &mut ItemContext<Self>,
        target: ViewTarget,
        config: ViewConfig,
    ) -> ViewSource {
        let source = TextureSource::empty(
            &(TextureSourceDescriptor {
                size: config.size,
                resolution: config.resolution,
                antialias: config.antialias,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT | config.usage,
                format: config.texture_format,
            }),
        );

        match target {
            ViewTarget::Surface(surface_target) => {
                let item =
                    cx.new_item(|cx| BackendRenderTarget::new(&cx.gpu, surface_target, &source));

                let handle = BackendRenderTargetHandle(item);

                ViewSource(source.replace(RenderTarget::from(handle)))
            }

            ViewTarget::Empty => ViewSource(source.replace(RenderTarget::Noop)),
        }
    }

    pub fn new(cx: &mut ItemContext<Self>, target: ViewTarget, config: ViewConfig) -> Self {
        let view = Self::create_view(cx, target, config);
        Self { view }
    }

    pub fn view(&self) -> &ViewSource {
        &self.view
    }

    pub fn replace_view(
        &mut self,
        cx: &mut ItemContext<Self>,
        target: ViewTarget,
        config: ViewConfig,
    ) {
        // create a new view
        self.view = Self::create_view(cx, target, config);
    }

    #[inline(always)]
    pub fn resize(&mut self, cx: &mut RenderContext, size: Size<u32>) {
        self.view.resize(cx, size);
    }

    #[inline(always)]
    pub fn set_resolution(&mut self, cx: &mut RenderContext, resolution: f32) {
        self.view.set_resolution(cx, resolution);
    }
}

pub trait ViewSystemExt: WithRenderContext {
    /// Access the view system, which is responsible for the primary screen \
    #[inline(always)]
    fn view_system<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&ViewSystem, &RenderContext) -> R,
    {
        self.rendering_context()
            .read_system(|view: &ViewSystem, cx: &RenderContext| f(view, cx))
    }

    /// Access the view system mutably, which is responsible for the primary screen \
    #[inline(always)]
    fn view_system_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ViewSystem, &mut ItemContext<ViewSystem>) -> R,
    {
        self.rendering_context_mut()
            .update_system(|view: &mut ViewSystem, cx| f(view, cx))
    }

    /// resize the primary screen to the given size
    #[inline(always)]
    fn resize(&mut self, size: Size<u32>) {
        self.view_system_mut(|view, cx| {
            view.resize(cx, size);
        });
    }

    /// set the resolution of the primary screen to the given value
    #[inline(always)]
    fn set_resolution(&mut self, resolution: f32) {
        self.view_system_mut(|view, cx| {
            view.set_resolution(cx, resolution);
        });
    }

    /// get the pixel size of the primary screen
    #[inline(always)]
    fn pixel_size(&self) -> Size<u32> {
        self.view_system(|view, _| view.view().pixel_size())
    }

    #[inline(always)]
    fn pixel_width(&self) -> u32 {
        self.view_system(|view, _| view.view().pixel_width())
    }

    #[inline(always)]
    fn pixel_height(&self) -> u32 {
        self.view_system(|view, _| view.view().pixel_height())
    }

    /// get the size of the primary screen
    #[inline(always)]
    fn screen_size(&self) -> Size<u32> {
        self.view_system(|view, _| view.view().size())
    }

    // replace the current view with a new one
    fn replace_view(&mut self, target: impl Into<ViewTarget>, config: ViewConfig) {
        self.view_system_mut(|view, cx| {
            view.replace_view(cx, target.into(), config);
        });
    }
}

impl<T> ViewSystemExt for T where T: WithRenderContext {}
