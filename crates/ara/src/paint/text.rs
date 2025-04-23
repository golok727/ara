use crate::{arc_string::ArcString, Font, FontStyle, FontWeight, Vec2, Zero};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextBaseline {
    #[default]
    Alphabetic,
    Top,
    Hanging,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
}

#[derive(Debug, Clone)]
pub struct Text {
    pub(crate) text: ArcString,
    pub(crate) font: Font,
    pub(crate) size: f32,
    pub(crate) pos: Vec2<f32>,
    pub(crate) align: TextAlign,
    pub(crate) word_spacing: f32,
    pub(crate) baseline: TextBaseline,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            pos: Vec2::zero(),
            text: ArcString::new_static(""),
            size: 16.0 * 2.0,
            font: Font {
                family: ArcString::new_static("Segoe UI"),
                weight: FontWeight::default(),
                style: FontStyle::default(),
            },
            align: Default::default(),
            baseline: Default::default(),
            word_spacing: f32::zero(),
        }
    }
}

impl Text {
    pub fn new(text: impl Into<ArcString>) -> Self {
        Self::default().text(text.into())
    }

    pub fn size_px(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn align(mut self, align: TextAlign) -> Self {
        self.align = align;
        self
    }

    pub fn baseline(mut self, baseline: TextBaseline) -> Self {
        self.baseline = baseline;
        self
    }

    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.pos.x = x;
        self.pos.y = y;
        self
    }

    pub fn text(mut self, text: ArcString) -> Self {
        self.text = text;
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn font_family(mut self, font_family: impl Into<ArcString>) -> Self {
        self.font.family = font_family.into();
        self
    }

    pub fn font_weight(mut self, font_weight: FontWeight) -> Self {
        self.font.weight = font_weight;
        self
    }

    pub fn font_style(mut self, font_style: FontStyle) -> Self {
        self.font.style = font_style;
        self
    }

    pub fn word_spacing(mut self, spacing_in_px: f32) -> Self {
        self.word_spacing = spacing_in_px;
        self
    }
}
