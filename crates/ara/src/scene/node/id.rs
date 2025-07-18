use std::fmt;
use std::sync::atomic::AtomicU32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SceneNodeId(pub(crate) u32);

impl SceneNodeIdentifier for SceneNodeId {
    fn id(&self) -> SceneNodeId {
        *self
    }
}

pub trait SceneNodeIdentifier {
    fn id(&self) -> SceneNodeId;
}

impl SceneNodeId {
    pub(crate) fn new() -> Self {
        static NEXT_ID: AtomicU32 = AtomicU32::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(id)
    }
}

impl fmt::Display for SceneNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NodeId({})", self.0)
    }
}
