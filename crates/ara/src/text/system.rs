use cosmic_text::{FontSystem as CosmisTextFontSystem, SwashCache};
use parking_lot::RwLock;

#[derive(Default)]
pub struct TextSystem(RwLock<TextSystemState>);

impl TextSystem {}

pub struct TextSystemState {
    pub font_system: CosmisTextFontSystem,
    pub swash_cache: SwashCache,
}

impl TextSystem {
    pub fn read<R>(&self, f: impl FnOnce(&TextSystemState) -> R) -> R {
        let state = self.0.read();
        f(&state)
    }

    pub fn write<R>(&self, f: impl FnOnce(&mut TextSystemState) -> R) -> R {
        let mut state = self.0.write();
        f(&mut state)
    }
}

impl Default for TextSystemState {
    fn default() -> Self {
        let font_system = CosmisTextFontSystem::new();
        Self {
            font_system,
            swash_cache: SwashCache::new(),
        }
    }
}
