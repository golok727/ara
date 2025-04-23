use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
};

mod system;
mod textarea;

pub use system::*;
// pub use textarea::*;

use crate::arc_string::ArcString;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GlyphImage {
    pub(crate) key: cosmic_text::CacheKey,
    pub(crate) is_emoji: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphId(pub(crate) usize);

pub type FontId = cosmic_text::fontdb::ID;

#[derive(Clone, Debug)]
pub struct FontMetrics {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Font {
    pub family: ArcString,
    pub weight: FontWeight,
    pub style: FontStyle,
}

impl Font {
    pub fn new(family: impl Into<ArcString>) -> Self {
        Self {
            family: family.into(),
            weight: FontWeight::default(),
            style: FontStyle::default(),
        }
    }

    pub fn bold(mut self) -> Self {
        self.weight = FontWeight::BOLD;
        self
    }

    pub fn italic(mut self) -> Self {
        self.style = FontStyle::Italic;
        self
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct FontWeight(pub f32);

impl Default for FontWeight {
    #[inline]
    fn default() -> FontWeight {
        FontWeight::NORMAL
    }
}

impl Hash for FontWeight {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_u32(u32::from_be_bytes(self.0.to_be_bytes()));
    }
}

impl Eq for FontWeight {}
impl FontWeight {
    pub const THIN: FontWeight = FontWeight(100.0);
    pub const EXTRA_LIGHT: FontWeight = FontWeight(200.0);
    pub const LIGHT: FontWeight = FontWeight(300.0);
    pub const NORMAL: FontWeight = FontWeight(400.0);
    pub const MEDIUM: FontWeight = FontWeight(500.0);
    pub const SEMIBOLD: FontWeight = FontWeight(600.0);
    pub const BOLD: FontWeight = FontWeight(700.0);
    pub const EXTRA_BOLD: FontWeight = FontWeight(800.0);
    pub const BLACK: FontWeight = FontWeight(900.0);
}

use cosmic_text::Style as CosmicStyle;
use cosmic_text::Weight as CosmicWeight;

impl From<FontWeight> for CosmicWeight {
    fn from(val: FontWeight) -> Self {
        match val {
            FontWeight::THIN => CosmicWeight::THIN,
            FontWeight::LIGHT => CosmicWeight::LIGHT,
            FontWeight::NORMAL => CosmicWeight::NORMAL,
            FontWeight::MEDIUM => CosmicWeight::MEDIUM,
            FontWeight::BOLD => CosmicWeight::BOLD,
            FontWeight::BLACK => CosmicWeight::BLACK,
            other => CosmicWeight(other.0 as u16),
        }
    }
}

impl From<FontStyle> for CosmicStyle {
    fn from(val: FontStyle) -> Self {
        match val {
            FontStyle::Normal => CosmicStyle::Normal,
            FontStyle::Italic => CosmicStyle::Italic,
            FontStyle::Oblique => CosmicStyle::Oblique,
        }
    }
}
