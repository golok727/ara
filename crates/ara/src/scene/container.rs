use parking_lot::RwLock;
use std::sync::Arc;

use crate::render::renderable::{DisplayObject, View};

use super::{
    AnyNode, ChildrenAccessMut, ChildrenStore, IntoSceneNode, ParentNode, RenderRoot, SceneNode,
    SceneNodeId, SceneNodeIdentifier, SceneNodeLike,
};

#[derive(Clone)]
pub struct Container {
    pub(crate) node: ContainerNode,
}

unsafe impl Send for Container {}
unsafe impl Sync for Container {}

impl SceneNodeIdentifier for Container {
    fn id(&self) -> SceneNodeId {
        self.node.id
    }
}

impl SceneNodeIdentifier for &Container {
    fn id(&self) -> SceneNodeId {
        self.node.id
    }
}

impl ParentNode for Container {
    fn extend(&mut self, nodes: impl Iterator<Item = AnyNode>) {
        self.node.inner.write().extend(nodes);
    }
}

impl ChildrenAccessMut for Container {
    fn with_children_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ChildrenStore) -> R,
    {
        self.node.inner.write().with_children_mut(f)
    }
}

impl Default for Container {
    fn default() -> Self {
        let id = SceneNodeId::new();
        let inner = Arc::new(RwLock::new(ContainerInner::default()));
        let node = ContainerNode::new(id, inner);
        Self { node }
    }
}

impl Container {
    pub fn new() -> Self {
        Default::default()
    }
}

impl View for Container {
    fn bounds(&self) -> ara_math::Rect<f32> {
        todo!()
    }

    fn contains_point(&self, _point: crate::Point) -> bool {
        todo!()
    }
}

impl RenderRoot for Container {
    type Node = ContainerNode;

    fn node(&self) -> &Self::Node {
        &self.node
    }
}

impl DisplayObject for Container {
    fn get_position(&self) -> crate::Point {
        todo!()
    }

    fn get_scale(&self) -> crate::Point {
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

impl IntoSceneNode for &Container {
    type Node = ContainerNode;

    fn into_scene_node(self) -> Self::Node {
        self.node.clone().into_scene_node()
    }
}

impl IntoSceneNode for ContainerNode {
    type Node = Self;
    fn into_scene_node(self) -> Self::Node {
        self
    }
}

#[derive(Debug, Default)]
pub(crate) struct ContainerInner {
    pub(crate) children: ChildrenStore,
}
unsafe impl Send for ContainerInner {}
unsafe impl Sync for ContainerInner {}

impl ParentNode for ContainerInner {
    fn extend(&mut self, nodes: impl Iterator<Item = AnyNode>) {
        self.children.extend(nodes);
    }
}

impl ChildrenAccessMut for ContainerInner {
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
pub struct ContainerNode {
    pub(crate) id: SceneNodeId,
    pub(crate) inner: Arc<RwLock<ContainerInner>>,
}

impl SceneNodeIdentifier for ContainerNode {
    fn id(&self) -> SceneNodeId {
        self.id
    }
}

impl SceneNode for ContainerNode {
    fn prepare(&self, render_context: &mut crate::render::RenderContext) {
        let inner = self.inner.read();
        for child in &inner.children.0 {
            child.prepare(render_context);
        }
    }

    fn paint<'encoder>(
        &self,
        pass: &mut wgpu::RenderPass<'encoder>,
        viewport: ara_math::Size<u32>,
        render_context: &mut crate::render::RenderContext,
    ) {
        let inner = self.inner.read();
        for child in &inner.children.0 {
            child.paint(pass, viewport, render_context);
        }
    }
}

impl ContainerNode {
    fn new(id: SceneNodeId, inner: Arc<RwLock<ContainerInner>>) -> Self {
        Self { id, inner }
    }
}
