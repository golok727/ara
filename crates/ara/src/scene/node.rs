pub mod id;

use ara_math::Size;
pub use id::*;
mod parent;
pub use parent::*;

use crate::render::RenderContext;

pub trait SceneNode: SceneNodeIdentifier + IntoSceneNode + 'static {
    fn prepare(&self, render_context: &mut RenderContext);
    fn paint<'encoder>(
        &self,
        pass: &mut wgpu::RenderPass<'encoder>,
        viewport: Size<u32>,
        render_context: &mut RenderContext,
    );
}

pub trait ParentNode: 'static {
    fn extend(&mut self, nodes: impl Iterator<Item = AnyNode>);

    fn child(&mut self, node: impl IntoSceneNode) -> &mut Self {
        let node = node.into_any_node();
        self.extend(std::iter::once(node));
        self
    }

    fn children(&mut self, nodes: impl IntoIterator<Item = impl IntoSceneNode>) -> &mut Self {
        let nodes = nodes.into_iter().map(|node| node.into_any_node());
        self.extend(nodes);
        self
    }
}

pub trait RenderRoot {
    type Node: SceneNode;
    fn node(&self) -> &Self::Node;
}

pub trait IntoSceneNode: Sized {
    type Node: SceneNode;
    fn into_scene_node(self) -> Self::Node;
    fn into_any_node(self) -> AnyNode
    where
        Self: Sized,
    {
        AnyNode::new(self.into_scene_node())
    }
}

pub(crate) struct NodeWrapper<T: SceneNode>(pub(crate) T);

impl<T: SceneNode> SceneNodeLike for NodeWrapper<T> {
    fn id(&self) -> SceneNodeId {
        self.0.id()
    }

    fn prepare(&self, render_context: &mut RenderContext) {
        self.0.prepare(render_context);
    }

    fn paint<'encoder>(
        &self,
        pass: &mut wgpu::RenderPass<'encoder>,
        viewport: Size<u32>,
        render_context: &mut RenderContext,
    ) {
        self.0.paint(pass, viewport, render_context);
    }
}

// dyn-compatible version
pub trait SceneNodeLike {
    fn id(&self) -> SceneNodeId;
    fn prepare(&self, render_context: &mut RenderContext);
    fn paint<'encoder>(
        &self,
        pass: &mut wgpu::RenderPass<'encoder>,
        viewport: Size<u32>,
        render_context: &mut RenderContext,
    );
}

// type erased node
pub struct AnyNode(pub(crate) Box<dyn SceneNodeLike>);

impl std::fmt::Debug for AnyNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("AnyNode").finish()
    }
}

impl AnyNode {
    pub fn new<T: SceneNode>(node: T) -> Self {
        Self(Box::new(NodeWrapper(node)))
    }
}

impl SceneNodeLike for AnyNode {
    fn id(&self) -> SceneNodeId {
        self.0.id()
    }

    fn prepare(&self, render_context: &mut RenderContext) {
        self.0.prepare(render_context);
    }

    fn paint<'encoder>(
        &self,
        pass: &mut wgpu::RenderPass<'encoder>,
        viewport: Size<u32>,
        render_context: &mut RenderContext,
    ) {
        self.0.paint(pass, viewport, render_context);
    }
}
