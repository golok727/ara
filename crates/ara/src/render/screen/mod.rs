#![deprecated(
    note = "The `ara::render::screen` module is deprecated and will be removed in favour of TextureSource"
)]

pub mod system;

use std::{ cell::RefCell, rc::Rc };

use super::{
    render_target::{ BackendRenderTargetHandle, RenderTarget },
    RenderContext,
    WithRenderContext,
};
use ara_math::Size;
use system::ScreenSystem;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScaledPixel(pub u32);
impl From<ScaledPixel> for u32 {
    fn from(val: ScaledPixel) -> Self {
        val.0
    }
}
impl ScaledPixel {
    #[inline]
    pub fn new(size: u32, scalar_factor: f32) -> Self {
        Self(((size as f32) * scalar_factor).round() as u32)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScreenConfig {
    pub size: Size<u32>,
    pub resolution: f32,
    pub antialias: bool,
    pub texture_format: wgpu::TextureFormat,
}

impl Default for ScreenConfig {
    fn default() -> Self {
        Self {
            size: Size::new(800, 600),
            resolution: 1.0,
            antialias: true,
            texture_format: wgpu::TextureFormat::Rgba8Unorm,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScreenId(pub(crate) usize);

#[derive(Debug, Clone)]
pub struct ScreenSpecs {
    pub(super) size: Size<u32>,
    pub(super) resolution: f32,
}

impl ScreenSpecs {
    #[inline]
    pub fn pixel_size(&self) -> Size<u32> {
        self.size.map(|s| ScaledPixel::new(s, self.resolution).into())
    }

    #[inline]
    pub fn pixel_width(&self) -> u32 {
        ScaledPixel::new(self.size.width, self.resolution).into()
    }

    #[inline]
    pub fn pixel_height(&self) -> u32 {
        ScaledPixel::new(self.size.height, self.resolution).into()
    }
}

#[derive(Debug, Clone)]
pub struct Screen {
    pub(super) id: ScreenId,
    pub(super) handle: BackendRenderTargetHandle,
    pub(super) specs: Rc<RefCell<ScreenSpecs>>,
}

impl Screen {
    pub(super) fn new(id: ScreenId, handle: BackendRenderTargetHandle, specs: ScreenSpecs) -> Self {
        Self {
            id,
            handle,
            specs: Rc::new(RefCell::new(specs)),
        }
    }
}

impl PartialEq for Screen {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Screen {}

impl std::hash::Hash for Screen {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

pub trait ScreenExt: WithRenderContext {
    #[must_use]
    #[inline]
    fn add_screen(
        &mut self,
        target: wgpu::SurfaceTarget<'static>,
        config: &ScreenConfig
    ) -> Screen {
        self.rendering_context_mut().update_system(|sys: &mut ScreenSystem, cx|
            sys.add(target, cx, config)
        )
    }

    fn remove_screen(&mut self, screen: Screen) {
        self.rendering_context_mut().update_system(|sys: &mut ScreenSystem, _| {
            sys.remove(screen);
        });
    }

    #[inline]
    fn resize_screen(&mut self, screen: &Screen, size: Size<u32>, resolution: f32) {
        screen.resize(self.rendering_context_mut(), size, resolution);
    }
}

impl<T: WithRenderContext> ScreenExt for T {}

impl From<&Screen> for RenderTarget {
    fn from(screen: &Screen) -> Self {
        RenderTarget::Backend(screen.handle.clone())
    }
}

impl Screen {
    pub fn size(&self) -> Size<u32> {
        self.specs.borrow().size
    }

    pub fn pixel_size(&self) -> Size<u32> {
        self.specs.borrow().pixel_size()
    }

    pub fn pixel_width(&self) -> u32 {
        self.specs.borrow().pixel_width()
    }

    pub fn pixel_height(&self) -> u32 {
        self.specs.borrow().pixel_height()
    }

    pub fn resolution(&self) -> f32 {
        self.specs.borrow().resolution
    }

    pub fn width(&self) -> u32 {
        self.specs.borrow().size.width
    }

    pub fn height(&self) -> u32 {
        self.specs.borrow().size.height
    }

    pub fn resize(&self, cx: &mut RenderContext, size: Size<u32>, resolution: f32) {
        cx.update_system(|sys: &mut ScreenSystem, cx| {
            sys.resize(self, cx, size, resolution);
        });
    }
}
