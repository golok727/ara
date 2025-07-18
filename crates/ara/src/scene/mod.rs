pub mod container;
pub mod graphics;
pub mod node;

pub use container::*;
pub use graphics::*;
pub use node::*;

use crate::render::Plugin;

use self::pipe::GraphicsPipe;

/// This plugin allows to render Graphics, Containers etc... This is registred by default
pub struct ScenePlugin;

impl Plugin for ScenePlugin {
    fn setup(&self, renderer: &mut crate::render::Renderer) {
        renderer.add_system(GraphicsContextSystem::new);
        renderer.add_pipe(GraphicsPipe::new);
    }
}
