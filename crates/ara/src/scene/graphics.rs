use crate::{
    math::{Corners, Rect},
    render::renderable::{DisplayObject, View},
    Color, LineCap, LineJoin, PathEvent, Point,
};
use ara_math::Size;
use parking_lot::RwLock;
use std::sync::Arc;

use super::{
    ChildrenAccessMut, ChildrenStore, ParentNode, RenderRoot, SceneNodeId, SceneNodeIdentifier,
    SceneNodeLike,
};

pub(crate) mod context;
pub(crate) mod context_system;
pub(crate) mod path;
pub(crate) mod pipe;

pub(crate) use context::GraphicsContext;
pub(crate) use context_system::{GpuGraphicsContext, GraphicsContextSystem};
use pipe::GraphicsPipe;

use super::{IntoSceneNode, SceneNode};

#[derive(Clone)]
pub struct Graphics {
    pub(crate) node: GraphicsNode,
}

impl Graphics {}

unsafe impl Send for Graphics {}
unsafe impl Sync for Graphics {}

impl Default for Graphics {
    fn default() -> Self {
        let id = SceneNodeId::new();

        let node = GraphicsNode {
            id,
            context: Arc::new(RwLock::new(GraphicsContext::new())),
            inner: Arc::new(RwLock::new(GraphicsInner::default())),
        };

        Self { node }
    }
}

impl ParentNode for Graphics {
    fn extend(&mut self, nodes: impl Iterator<Item = super::AnyNode>) {
        self.node.inner.write().extend(nodes)
    }
}

impl ChildrenAccessMut for Graphics {
    fn with_children_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ChildrenStore) -> R,
    {
        self.node.inner.write().with_children_mut(f)
    }
}

impl SceneNodeIdentifier for Graphics {
    fn id(&self) -> SceneNodeId {
        self.node.id
    }
}
impl SceneNodeIdentifier for &Graphics {
    fn id(&self) -> SceneNodeId {
        self.node.id
    }
}

impl RenderRoot for Graphics {
    type Node = GraphicsNode;

    fn node(&self) -> &Self::Node {
        &self.node
    }
}

impl View for Graphics {
    fn bounds(&self) -> Rect<f32> {
        todo!()
    }

    fn contains_point(&self, _point: Point) -> bool {
        todo!()
    }
}

impl DisplayObject for Graphics {
    fn get_position(&self) -> Point {
        todo!()
    }

    fn get_scale(&self) -> Point {
        todo!()
    }

    fn get_rotation(&self) -> f32 {
        todo!()
    }

    fn renderable(&self) -> bool {
        todo!()
    }

    fn visible(&self) -> bool {
        todo!()
    }

    fn alpha(&self) -> f32 {
        todo!()
    }
}

impl IntoSceneNode for &Graphics {
    type Node = GraphicsNode;

    fn into_scene_node(self) -> Self::Node {
        self.node.clone().into_scene_node()
    }
}

impl Graphics {
    pub fn cx<T>(&self, f: impl FnOnce(&GraphicsContext) -> T) -> T {
        f(&self.node.context.read())
    }

    pub fn cx_mut<T>(&self, f: impl FnOnce(&mut GraphicsContext) -> T) -> T {
        f(&mut self.node.context.write())
    }
}

impl Graphics {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clip(&mut self, rect: Rect<f32>) -> &mut Self {
        self.node.context.write().set_clip(rect);
        self
    }

    pub fn get_clip(&self) -> Rect<f32> {
        self.node.context.read().get_clip()
    }

    pub fn reset_clip(&mut self) -> &mut Self {
        self.node.context.write().reset_clip();
        self
    }

    /// Reset the current state to default values
    pub fn reset(&mut self) -> &mut Self {
        self.node.context.write().reset();
        self
    }

    pub fn reset_transform(&mut self) -> &mut Self {
        self.node.context.write().reset_transform();
        self
    }

    pub fn get_line_width(&self) -> u32 {
        self.node.context.read().get_line_width()
    }

    pub fn line_width(&mut self, line_width: u32) -> &mut Self {
        self.node.context.write().set_line_width(line_width);
        self
    }

    pub fn get_line_join(&self) -> LineJoin {
        self.node.context.read().get_line_join()
    }

    pub fn line_join(&mut self, line_join: LineJoin) -> &mut Self {
        self.node.context.write().set_line_join(line_join);
        self
    }

    pub fn get_line_cap(&self) -> LineCap {
        self.node.context.read().get_line_cap()
    }

    pub fn line_cap(&mut self, line_cap: LineCap) -> &mut Self {
        self.node.context.write().set_line_cap(line_cap);
        self
    }

    pub fn save(&mut self) -> &mut Self {
        self.node.context.write().save();
        self
    }

    pub fn restore(&mut self) -> &mut Self {
        self.node.context.write().restore();
        self
    }

    /// Applies a translation transformation to the graphics context.
    pub fn translate(&mut self, dx: f32, dy: f32) -> &mut Self {
        self.node.context.write().translate(dx, dy);
        self
    }

    /// Applies a scaling transformation to the graphics context.
    /// If you want to scale this entire graphics use `scale`
    pub fn scale(&mut self, sx: f32, sy: f32) -> &mut Self {
        self.node.context.write().scale(sx, sy);
        self
    }

    pub fn rotate(&mut self, angle: f32) -> &mut Self {
        self.node.context.write().rotate(angle);
        self
    }

    pub fn path<T>(&mut self, path: T) -> &mut Self
    where
        T: IntoIterator<Item = PathEvent>,
    {
        self.node.context.write().path(path);
        self
    }

    pub fn rect(&mut self, rect: impl Into<Rect<f32>>) -> &mut Self {
        self.node.context.write().rect(rect.into());
        self
    }

    pub fn round_rect(
        &mut self,
        rect: impl Into<Rect<f32>>,
        radii: impl Into<Corners<f32>>,
    ) -> &mut Self {
        self.node
            .context
            .write()
            .round_rect(rect.into(), radii.into());
        self
    }

    pub fn circle(&mut self, center: impl Into<Point>, radius: f32) -> &mut Self {
        self.node.context.write().circle(center.into(), radius);
        self
    }

    pub fn clear(&self) -> &Self {
        self.node.context.write().clear();
        self
    }

    pub fn fill(&mut self, color: impl Into<Color>) -> &mut Self {
        self.node.context.write().fill(color);
        self
    }

    pub fn stroke(&mut self, color: impl Into<Color>) -> &mut Self {
        self.node.context.write().stroke(color);
        self
    }
}

#[derive(Default)]
pub(crate) struct GraphicsInner {
    // todo view stuff
    pub(crate) children: ChildrenStore,
}

unsafe impl Send for GraphicsInner {}
unsafe impl Sync for GraphicsInner {}

impl ParentNode for GraphicsInner {
    fn extend(&mut self, nodes: impl Iterator<Item = super::AnyNode>) {
        self.children.extend(nodes);
    }
}

impl ChildrenAccessMut for GraphicsInner {
    fn with_children_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ChildrenStore) -> R,
    {
        f(&mut self.children)
    }
}

// ---------------------
// Node
// ---------------------
#[derive(Clone)]
pub struct GraphicsNode {
    pub(crate) id: SceneNodeId,
    pub(crate) context: Arc<RwLock<GraphicsContext>>,
    pub(crate) inner: Arc<RwLock<GraphicsInner>>,
}

unsafe impl Send for GraphicsNode {}
unsafe impl Sync for GraphicsNode {}

impl SceneNodeIdentifier for GraphicsNode {
    fn id(&self) -> SceneNodeId {
        self.id
    }
}

impl SceneNode for GraphicsNode {
    fn prepare(&self, render_context: &mut crate::render::RenderContext) {
        {
            let context = self.context.read();
            render_context.update_pipe(|pipe: &mut GraphicsPipe, cx| {
                pipe.prepare(cx, &context);
            });
        }
        let inner = self.inner.read();
        for child in &inner.children.0 {
            child.prepare(render_context);
        }
    }
    fn paint<'encoder>(
        &self,
        pass: &mut wgpu::RenderPass<'encoder>,
        viewport: Size<u32>,
        render_context: &mut crate::render::RenderContext,
    ) {
        {
            let context = self.context.read();
            render_context.update_pipe(|pipe: &mut GraphicsPipe, cx| {
                pipe.execute(pass, viewport, cx, &context);
            });
        }

        let inner = self.inner.read();
        for child in &inner.children.0 {
            child.paint(pass, viewport, render_context);
        }
    }
}

impl IntoSceneNode for GraphicsNode {
    type Node = Self;

    fn into_scene_node(self) -> Self::Node {
        self
    }
}
