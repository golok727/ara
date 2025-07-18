use ara_math::Size;

use crate::render::{
    render_target::{ BackendRenderTarget, BackendRenderTargetHandle, RenderTargetConfig },
    systems::System,
    ItemContext,
    ItemManager,
    RenderContext,
};

use super::{ ScaledPixel, Screen, ScreenConfig, ScreenId, ScreenSpecs };

/* The screens handled by ara! */
pub struct ScreenSystem {
    screens: Vec<Screen>,
}

impl System for ScreenSystem {
    fn init(&mut self, _cx: &mut RenderContext) where Self: Sized {}
}

impl ScreenSystem {
    pub fn new(_: &mut ItemContext<Self>) -> Self {
        Self {
            screens: Default::default(),
        }
    }

    fn has(&self, screen_id: ScreenId) -> bool {
        self.screens.get(screen_id.0).is_some()
    }

    pub fn resize(
        &self,
        screen: &Screen,
        cx: &mut RenderContext,
        size: Size<u32>,
        resolution: f32
    ) {
        if !self.has(screen.id) {
            return;
        }

        let pixel_size = size
            .map(|s| s as f32)
            .scale(resolution)
            .floor()
            .map(|s| s as u32);

        let _ = cx.update_item(&screen.handle, |target, cx| {
            target.resize(&cx.gpu.device, pixel_size.width, pixel_size.height);
        });

        let mut specs = screen.specs.borrow_mut();
        specs.size = size;
        specs.resolution = resolution;
    }

    pub fn remove(&mut self, screen: Screen) {
        let ix = screen.id.0;

        if ix < self.screens.len() && self.screens[ix].id == screen.id {
            self.screens.swap_remove(ix);
        }
    }

    pub fn add<C: ItemManager>(
        &mut self,
        target: wgpu::SurfaceTarget<'static>,
        cx: &mut C,
        config: &ScreenConfig
    ) -> Screen {
        let id = ScreenId(self.screens.len());

        let size = config.size;
        let pixel_size: Size<u32> = size.map(|s| ScaledPixel::new(s, config.resolution).into());

        let target_config = RenderTargetConfig {
            width: pixel_size.width.max(1),
            height: pixel_size.height.max(1),
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            antialias: config.antialias,
        };

        let handle = cx.new_item(|cx| BackendRenderTarget::new(&cx.gpu, target, &target_config));

        let handle = BackendRenderTargetHandle(handle);

        let screen = Screen::new(id, handle, ScreenSpecs {
            size,
            resolution: config.resolution,
        });

        self.screens.push(screen.clone());

        screen
    }
}
