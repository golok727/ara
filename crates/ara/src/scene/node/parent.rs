use core::fmt;

use super::{AnyNode, IntoSceneNode, ParentNode, SceneNodeIdentifier, SceneNodeLike};

#[derive(Default)]
pub struct ChildrenStore(pub(crate) Vec<AnyNode>);
impl ChildrenStore {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for ChildrenStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ChildrenContainer")
            .field(&self.0.len())
            .finish()
    }
}

impl ParentNode for ChildrenStore {
    fn extend(&mut self, nodes: impl Iterator<Item = AnyNode>) {
        self.0.extend(nodes)
    }
}

pub trait ChildrenAccessMut {
    fn with_children_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ChildrenStore) -> R;
}

// Generic implementation for anything that has direct access to ChildrenStore
impl ChildrenAccessMut for ChildrenStore {
    fn with_children_mut<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut ChildrenStore) -> R,
    {
        f(self)
    }
}

// Default implementation for ParentNode providing child manipulation methods
pub trait ChildrenController: ChildrenAccessMut + ParentNode {
    fn clear_children(&mut self) {
        self.with_children_mut(|store| store.0.clear());
    }

    fn replace_children(&mut self, nodes: impl IntoIterator<Item = impl IntoSceneNode>) {
        self.with_children_mut(|store| {
            store.0.clear();
            let nodes = nodes.into_iter().map(|node| node.into_any_node());
            store.0.extend(nodes);
        });
    }

    fn remove_children<I, N>(&mut self, nodes: I)
    where
        I: IntoIterator<Item = N>,
        N: SceneNodeIdentifier,
    {
        self.with_children_mut(|store| {
            let ids: ahash::HashSet<_> = nodes.into_iter().map(|n| n.id()).collect();

            store.0.retain(|c| !ids.contains(&c.id()));
        });
    }

    fn remove_child(&mut self, node: impl SceneNodeIdentifier) {
        let id = node.id();
        self.with_children_mut(|store| {
            store.0.retain(|child| child.id() != id);
        });
    }
}
// Blanket implementation for any type that implements ChildrenAccessMut and ParentNode
impl<T: ChildrenAccessMut + ParentNode> ChildrenController for T {}
