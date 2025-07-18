use std::any::Any;

use super::Renderer;

pub trait Plugin: Any {
    /// called as soon as added
    fn setup(&self, renderer: &mut Renderer);

    /// called on renderer initialization at this point all the plugins will be loaded
    fn finish(&self, _: &mut Renderer) {}

    /// an unique identifier for the plugin
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// If this plugin is already registered can it override ? default: false
    fn can_override(&self) -> bool {
        false
    }
}
