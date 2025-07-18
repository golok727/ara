use crate::{
    render::{
        systems::{GeometryHandle, System},
        ItemContext, RenderCommand,
    },
    scene::context::{GraphicsContext, GraphicsContextId},
};

pub struct GraphicsContextSystem {
    gpu_contexts: ahash::HashMap<GraphicsContextId, GpuGraphicsContext>,
}

impl GraphicsContextSystem {
    pub fn new(_: &mut ItemContext<Self>) -> Self {
        Self {
            gpu_contexts: Default::default(),
        }
    }
}

impl System for GraphicsContextSystem {
    fn init(&mut self, _cx: &mut crate::render::RenderContext) {}
}

#[derive(Debug)]
pub struct GpuGraphicsContext {
    pub(crate) geometry_handle: GeometryHandle,
    pub(crate) commands: Vec<RenderCommand>,
}

impl GpuGraphicsContext {
    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn add_command(&mut self, command: RenderCommand) {
        self.commands.push(command);
    }
}

impl GpuGraphicsContext {
    pub fn new(geometry_handle: GeometryHandle) -> Self {
        Self {
            geometry_handle,
            commands: Default::default(),
        }
    }
}

impl GraphicsContextSystem {
    pub fn get_cx(&self, context: &GraphicsContext) -> Option<&GpuGraphicsContext> {
        self.gpu_contexts.get(&context.id())
    }

    pub fn get_or_init_cx(
        &mut self,
        context: &GraphicsContext,
        insert: impl FnOnce() -> GpuGraphicsContext,
    ) -> &mut GpuGraphicsContext {
        self.gpu_contexts.entry(context.id()).or_insert_with(insert)
    }
}
