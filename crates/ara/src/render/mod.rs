// This is the new renderer module for Ara;
// unlike ara::Canvas This is a retained mode renderer
pub mod pipes;
pub mod systems;
pub mod texture;
pub mod view;

use crate::gpu;
use crate::scene::RenderRoot;
use crate::scene::SceneNode;
use crate::scene::ScenePlugin;
use crate::Slot;
use crate::Subscription;
use ahash::HashSet;
use ara_math::Size;
use pipes::{PipeCollection, RenderPipe};
use render_target::RenderTarget;
use render_target::RenderTargetSystem;
use renderable::{DisplayObject, Renderable, View};
use runner::{RenderExecContext, RenderRunners};
use systems::EncoderSystem;
pub use view::{ViewConfig, ViewSystem, ViewSystemExt, ViewTarget};

use systems::{GeometryHandle, RenderBufferRange, System, SystemCollection};

pub mod context;
pub use context::*;

pub mod item_map;
pub use item_map::*;

pub mod plugin;
use crate::Color;
pub use plugin::*;

pub mod render_target;
pub mod renderable;
pub mod runner;

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTargetView {
    pub(crate) target: RenderTarget,
    pub(crate) pixel_size: Size<u32>,
    pub(crate) screen_size: Size<u32>,
}

pub struct RenderTo {
    pub target: ViewTarget,
    pub config: ViewConfig,
}

impl Default for RenderTo {
    fn default() -> Self {
        Self {
            target: ViewTarget::Empty,
            config: ViewConfig::default(),
        }
    }
}

#[derive(Default)]
pub struct RendererSpecification {
    pub render_to: RenderTo,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
enum RenderState {
    #[default]
    Uninitialized,
    Initialized,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PluginState {
    Adding,
    Finalizing,
    Complete,
}

impl RenderState {
    fn is_initialized(&self) -> bool {
        matches!(self, Self::Initialized)
    }
}

pub struct Renderer {
    gpu: gpu::Context,

    context: RenderContext,

    // todo move to background system
    last_clear_color: Color,

    plugins: Vec<Box<dyn Plugin>>,
    plugin_name_hash: HashSet<&'static str>,
    plugin_state: PluginState,

    renderer_state: RenderState,
}

impl Renderer {
    fn set_default_configuration(this: &mut Self, specs: RendererSpecification) {
        this.add_plugins(DefaultPlugins)
            .add_system(|cx| ViewSystem::new(cx, specs.render_to.target, specs.render_to.config))
            .add_plugins(ScenePlugin);
    }

    pub fn new(gpu: &gpu::Context, specs: RendererSpecification) -> Self {
        let context = RenderContext::new(gpu.clone());

        let mut this = Self {
            gpu: gpu.clone(),
            context,

            last_clear_color: Color::BLACK,

            plugins: Default::default(),
            plugin_state: PluginState::Adding,
            plugin_name_hash: HashSet::default(),

            renderer_state: RenderState::Uninitialized,
        };
        Self::set_default_configuration(&mut this, specs);
        this
    }

    pub fn on_init<F>(&self, f: F)
    where
        F: Fn(&mut RenderContext) + 'static,
    {
        let Some(slot) = self.context.init_slot.as_ref() else {
            return;
        };
        slot.add(Box::new(f)).detach();
    }

    pub fn add_plugins<P: Plugin + 'static>(&mut self, plugin: P) -> &mut Self {
        self.add_boxed_plugin(Box::new(plugin));
        self
    }

    pub(crate) fn add_boxed_plugin(&mut self, plugin: Box<dyn Plugin>) {
        if matches!(
            self.plugin_state,
            PluginState::Complete | PluginState::Finalizing
        ) {
            panic!("Cannot add plugins after initializing app");
        }

        let plugin_name = plugin.name();

        if self.plugin_name_hash.contains(&plugin_name) {
            panic!("Plugin with name {} already exists", plugin_name);
        }

        // preserve the insertion index
        let index = self.plugins.len();
        self.plugins.push(Box::new(PlaceholderPlugin));
        plugin.setup(self);
        self.plugin_name_hash.insert(plugin_name);

        self.plugins[index] = plugin;
    }

    pub fn finish_plugin_stuff(&mut self) {
        self.plugin_state = PluginState::Finalizing;

        let plugins = std::mem::take(&mut self.plugins);

        for plugin in &plugins {
            plugin.finish(self);
        }

        self.plugins = plugins;

        self.plugin_state = PluginState::Complete;
    }

    pub fn init(&mut self) -> &mut Self {
        debug_assert!(
            self.renderer_state != RenderState::Initialized,
            "Renderer is already initialized"
        );

        self.finish_plugin_stuff();

        self.context.init();

        self.renderer_state = RenderState::Initialized;
        self
    }

    pub fn gpu(&self) -> &gpu::Context {
        &self.gpu
    }

    pub fn add_system<S: System + 'static>(
        &mut self,
        build: impl FnOnce(&mut ItemContext<S>) -> S,
    ) -> &mut Self {
        assert!(
            !self.renderer_state.is_initialized(),
            "Cant add systems after initializing Renderer"
        );
        self.context.add_system(build);

        self
    }

    pub fn add_pipe<P: RenderPipe + 'static>(
        &mut self,
        build: impl FnOnce(&mut ItemContext<P>) -> P,
    ) -> &mut Self {
        assert!(
            !self.renderer_state.is_initialized(),
            "Cant add pipes after initializing Renderer"
        );

        self.context.add_pipe(build);

        self
    }

    pub fn add_runner(
        &self,
        runner: RenderRunner,
        callback: impl (Fn(&mut RenderExecContext) -> anyhow::Result<()>) + 'static,
    ) -> Subscription {
        assert!(
            !self.renderer_state.is_initialized(),
            "Cant add runners after initializing Renderer"
        );

        self.context.add_runner(runner, callback)
    }

    pub fn render<R>(&mut self, root: &R, options: impl Into<RenderOptions>)
    where
        R: RenderRoot + DisplayObject,
    {
        debug_assert!(
            self.renderer_state.is_initialized(),
            "Renderer is not initialized please call Renderer::init() before rendering"
        );

        let options: RenderOptions = options.into();
        let renderable = RootRenderable { root };

        let view = options.view.unwrap_or_else(|| {
            self.context.read_system(|sys: &ViewSystem, _| {
                let view = sys.view();
                RenderTargetView {
                    target: view.source.clone(),
                    pixel_size: view.pixel_size(),
                    screen_size: view.size(),
                }
            })
        });
        let clear_color = options.clear_color.unwrap_or(self.last_clear_color);
        self.last_clear_color = clear_color;

        self.context.runners.start.clone().emit(|callback| {
            let mut cx = RenderExecContext {
                view: &view,
                kind: RenderRunner::Start,
                renderable: &renderable,
                cx: &mut self.context,
                clear_color,
            };

            if let Err(err) = callback(&mut cx) {
                log::error!("Error in start callback: {}", err);
            }
        });

        self.context.runners.prerender.clone().emit(|callback| {
            let mut cx = RenderExecContext {
                view: &view,
                kind: RenderRunner::PreRender,
                renderable: &renderable,
                cx: &mut self.context,
                clear_color,
            };

            if let Err(err) = callback(&mut cx) {
                log::error!("Error in prepare callback: {}", err);
            }
        });

        self.context.runners.render.clone().emit(|callback| {
            let mut cx = RenderExecContext {
                view: &view,
                kind: RenderRunner::Render,
                renderable: &renderable,
                cx: &mut self.context,
                clear_color,
            };

            if let Err(err) = callback(&mut cx) {
                log::error!("Error in render callback: {}", err);
            }
        });

        self.context.runners.postrender.clone().emit(|callback| {
            let mut cx = RenderExecContext {
                view: &view,
                kind: RenderRunner::PostRender,
                renderable: &renderable,
                cx: &mut self.context,
                clear_color,
            };

            if let Err(err) = callback(&mut cx) {
                log::error!("Error in postrender callback: {}", err);
            }
        });

        self.context.runners.finish.clone().emit(|callback| {
            let mut cx = RenderExecContext {
                view: &view,
                kind: RenderRunner::Finish,
                renderable: &renderable,
                cx: &mut self.context,
                clear_color,
            };

            if let Err(err) = callback(&mut cx) {
                log::error!("Error in finish callback: {}", err);
            }
        });
    }
}

impl ItemManager for Renderer {
    fn new_item<T: 'static>(&mut self, create: impl FnOnce(&mut ItemContext<T>) -> T) -> Item<T> {
        self.context.new_item(create)
    }

    fn update_item<T: 'static, R>(
        &mut self,
        handle: &Item<T>,
        update: impl FnOnce(&mut T, &mut ItemContext<T>) -> R,
    ) -> anyhow::Result<R> {
        self.context.update_item(handle, update)
    }

    fn read_item<T: 'static, R>(
        &self,
        handle: &Item<T>,
        read: impl FnOnce(&T, &RenderContext) -> R,
    ) -> anyhow::Result<R> {
        self.context.read_item(handle, read)
    }
}

impl WithRenderContext for Renderer {
    fn rendering_context(&self) -> &RenderContext {
        &self.context
    }

    fn rendering_context_mut(&mut self) -> &mut RenderContext {
        &mut self.context
    }
}

pub enum RenderRunner {
    Start,
    PreRender,
    Render,
    PostRender,
    Finish,
}

type InitCallback = Box<dyn Fn(&mut RenderContext) + 'static>;

pub struct RenderContext {
    pub(crate) gpu: gpu::Context,
    pub(crate) runners: RenderRunners,
    pub(crate) items: ItemMap,
    pub(crate) pipes_collection: PipeCollection,
    pub(crate) systems_collection: SystemCollection,
    init_slot: Option<Slot<InitCallback>>,
}

impl RenderContext {
    pub fn new(gpu: gpu::Context) -> Self {
        Self {
            pipes_collection: Default::default(),
            systems_collection: Default::default(),
            runners: Default::default(),
            items: ItemMap::new(),
            init_slot: Some(Default::default()),
            gpu,
        }
    }
}

impl WithRenderContext for RenderContext {
    fn rendering_context_mut(&mut self) -> &mut RenderContext {
        self
    }

    fn rendering_context(&self) -> &RenderContext {
        self
    }
}

impl RenderContext {
    pub fn gpu(&self) -> &gpu::Context {
        &self.gpu
    }

    fn add_system<S: System + 'static>(&mut self, build: impl FnOnce(&mut ItemContext<S>) -> S) {
        let handle = self.new_item::<S>(|cx| build(cx));
        self.systems_collection.add(handle);
    }

    fn add_pipe<P: RenderPipe + 'static>(&mut self, build: impl FnOnce(&mut ItemContext<P>) -> P) {
        let handle = self.new_item::<P>(|cx| build(cx));
        self.pipes_collection.add(handle);
    }

    pub fn read_pipe<P: RenderPipe + 'static, R>(
        &mut self,
        read: impl FnOnce(&P, &RenderContext) -> R,
    ) -> R {
        let handle: Item<P> = self
            .pipes_collection
            .get_handle()
            .unwrap_or_else(|| panic!("Pipe {} not registered", std::any::type_name::<P>()));

        handle.read(self, read).expect("Pipe Released")
    }

    pub fn update_pipe<P: RenderPipe + 'static, R>(
        &mut self,
        update: impl FnOnce(&mut P, &mut ItemContext<P>) -> R,
    ) -> R {
        let handle: Item<P> = self
            .pipes_collection
            .get_handle()
            .unwrap_or_else(|| panic!("Pipe {} not registered", std::any::type_name::<P>()));

        handle.update(self, update).expect("Pipe Released")
    }

    pub fn read_system<S: System + 'static, R>(
        &self,
        read: impl FnOnce(&S, &RenderContext) -> R,
    ) -> R {
        let handle: Item<S> = self
            .systems_collection
            .get_handle()
            .unwrap_or_else(|| panic!("System {} not registered", std::any::type_name::<S>()));

        handle.read(self, read).expect("System released")
    }

    pub fn update_system<S: System + 'static, R>(
        &mut self,
        update: impl FnOnce(&mut S, &mut ItemContext<S>) -> R,
    ) -> R {
        let handle: Item<S> = self
            .systems_collection
            .get_handle()
            .unwrap_or_else(|| panic!("System {} not registered", std::any::type_name::<S>()));
        handle.update(self, update).expect("System released")
    }

    fn init(&mut self) {
        SystemCollection::init(self);
        PipeCollection::init(self);
        let Some(init_slot) = self.init_slot.take() else {
            return;
        };
        init_slot.emit(|callback| callback(self));
    }

    pub fn add_runner(
        &self,
        runner: RenderRunner,
        callback: impl (Fn(&mut RenderExecContext) -> anyhow::Result<()>) + 'static,
    ) -> Subscription {
        match runner {
            RenderRunner::Start => self.runners.start.add(Box::new(callback)),
            RenderRunner::PreRender => self.runners.prerender.add(Box::new(callback)),
            RenderRunner::Render => self.runners.render.add(Box::new(callback)),
            RenderRunner::PostRender => self.runners.postrender.add(Box::new(callback)),
            RenderRunner::Finish => self.runners.finish.add(Box::new(callback)),
        }
    }
}

impl ItemManager for RenderContext {
    fn new_item<T: 'static>(&mut self, create: impl FnOnce(&mut ItemContext<T>) -> T) -> Item<T> {
        let slot = self.items.reserve();
        let mut item_cx = ItemContext::new(slot.downgrade(), self);
        let item = create(&mut item_cx);

        self.items.insert(slot, item)
    }

    fn update_item<T: 'static, R>(
        &mut self,
        handle: &Item<T>,
        update: impl FnOnce(&mut T, &mut ItemContext<T>) -> R,
    ) -> anyhow::Result<R> {
        let mut lease = self.items.lease(handle);
        let mut cx = ItemContext::new(handle.downgrade(), self);
        let res = update(&mut lease, &mut cx);
        self.items.end_lease(lease);
        Ok(res)
    }

    fn read_item<T: 'static, R>(
        &self,
        handle: &Item<T>,
        read: impl FnOnce(&T, &RenderContext) -> R,
    ) -> anyhow::Result<R> {
        let item = self.items.read(handle);
        Ok(read(item, self))
    }
}

#[derive(Debug, Clone)]
pub enum RenderCommand {
    SetScissor {
        rect: crate::Rect<f32>,
    },
    DrawIndexed {
        geometry_handle: GeometryHandle,
        render_buffer_slice: RenderBufferRange,
    },
}

impl RenderCommand {
    pub fn set_scissor(rect: crate::Rect<f32>) -> Self {
        Self::SetScissor { rect }
    }

    pub fn draw_indexed(
        geometry_handle: GeometryHandle,
        render_buffer_slice: RenderBufferRange,
    ) -> Self {
        Self::DrawIndexed {
            geometry_handle,
            render_buffer_slice,
        }
    }
}

struct PlaceholderPlugin;

impl Plugin for PlaceholderPlugin {
    fn setup(&self, _: &mut Renderer) {
        // noop
    }
}

struct RootRenderable<'a, R>
where
    R: RenderRoot + DisplayObject,
{
    root: &'a R,
}

impl<'a, R> Renderable for RootRenderable<'a, R>
where
    R: RenderRoot + DisplayObject,
{
    fn prepare(&self, render_context: &mut crate::render::RenderContext) {
        self.root.node().prepare(render_context);
    }

    fn paint<'encoder>(
        &self,
        pass: &mut wgpu::RenderPass<'encoder>,
        viewport: ara_math::Size<u32>,
        render_context: &mut crate::render::RenderContext,
    ) {
        self.root.node().paint(pass, viewport, render_context);
    }
}

impl<'a, R> View for RootRenderable<'a, R>
where
    R: RenderRoot + DisplayObject,
{
    fn bounds(&self) -> ara_math::Rect<f32> {
        self.root.bounds()
    }

    fn contains_point(&self, point: crate::Point) -> bool {
        self.root.contains_point(point)
    }
}

impl<'a, R> DisplayObject for RootRenderable<'a, R>
where
    R: RenderRoot + DisplayObject,
{
    fn get_position(&self) -> crate::Point {
        self.root.get_position()
    }

    fn get_scale(&self) -> crate::Point {
        self.root.get_scale()
    }

    fn get_rotation(&self) -> f32 {
        self.root.get_rotation()
    }

    fn renderable(&self) -> bool {
        self.root.renderable()
    }

    fn visible(&self) -> bool {
        self.root.visible()
    }

    fn alpha(&self) -> f32 {
        self.root.alpha()
    }
}

pub(crate) struct DefaultPlugins;
impl Plugin for DefaultPlugins {
    fn setup(&self, renderer: &mut Renderer) {
        use renderable::RenderableSystem;
        use systems::{GeometrySystem, GlobalUniformSystem, HelloSystem};

        renderer
            .add_system(|_| HelloSystem)
            .add_system(EncoderSystem::new)
            .add_system(GlobalUniformSystem::new)
            .add_system(GeometrySystem::new)
            .add_system(RenderTargetSystem::new)
            .add_system(RenderableSystem::new);
    }
}

pub trait WithRenderContext {
    fn rendering_context(&self) -> &RenderContext;
    fn rendering_context_mut(&mut self) -> &mut RenderContext;
}

#[derive(Default, Clone, Debug)]
pub struct RenderOptions {
    // if none color from background system will be used color will be used
    pub clear_color: Option<Color>,
    // which view to render to ? if none primary view will be used
    pub view: Option<RenderTargetView>,
}

impl RenderOptions {
    pub fn clear_color(mut self, color: Color) -> Self {
        self.clear_color = Some(color);
        self
    }

    pub fn view(mut self, view: impl Into<RenderTargetView>) -> Self {
        self.view = Some(view.into());
        self
    }
}

impl From<()> for RenderOptions {
    fn from(_: ()) -> Self {
        Self::default()
    }
}

impl From<Color> for RenderOptions {
    fn from(clear_color: Color) -> Self {
        Self {
            clear_color: Some(clear_color),
            view: None,
        }
    }
}

impl<T> From<T> for RenderOptions
where
    T: Into<RenderTargetView>,
{
    fn from(view: T) -> Self {
        Self {
            clear_color: None,
            view: Some(view.into()),
        }
    }
}
