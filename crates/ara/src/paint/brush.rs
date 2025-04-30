use std::ops::{Deref, DerefMut};

use ara_math::{Corners, Rect};

use crate::{
    path::{Contour, Point},
    Canvas, PathBuilder, Polygon,
};

use super::Color;

/// Represents a brush used for drawing operations, which includes properties for fill style, stroke style, and anti-aliasing.
#[derive(Debug, Clone, PartialEq)]
pub struct Brush {
    pub(crate) fill_style: FillStyle,
    pub(crate) stroke_style: StrokeStyle,
    pub(crate) antialias: bool,
}

impl Default for Brush {
    /// Creates a default brush with transparent fill and stroke, and anti-aliasing disabled.
    fn default() -> Self {
        Self {
            fill_style: FillStyle {
                color: Color::TRANSPARENT,
            },
            stroke_style: StrokeStyle {
                color: Color::TRANSPARENT,
                ..Default::default()
            },
            antialias: false,
        }
    }
}

impl Brush {
    pub fn filled(fill_color: Color) -> Self {
        Self {
            fill_style: FillStyle { color: fill_color },
            ..Default::default()
        }
    }
    /// Returns whether anti-aliasing is enabled for the brush.
    pub fn is_antialias(&self) -> bool {
        self.antialias
    }

    /// Enables or disables anti-aliasing for the brush.
    ///
    /// # Arguments
    ///
    /// * `enable` - A boolean value to enable (true) or disable (false) anti-aliasing.
    pub fn antialias(mut self, enable: bool) -> Self {
        self.antialias = enable;
        self
    }

    /// Gets the current fill color of the brush.
    pub fn get_fill_color(&self) -> Color {
        self.fill_style.color
    }

    /// Sets the fill color of the brush.
    ///
    /// # Arguments
    ///
    /// * `color` - The new fill color to be applied.
    pub fn fill_color(mut self, color: Color) -> Self {
        self.fill_style.color = color;
        self
    }

    pub fn reset_fill(mut self) -> Self {
        self.fill_style = Default::default();
        self
    }

    pub fn reset_stroke(mut self) -> Self {
        self.stroke_style = Default::default();
        self
    }

    pub fn no_fill(mut self) -> Self {
        self.fill_style.color = Color::TRANSPARENT;
        self
    }

    pub fn no_stroke(mut self) -> Self {
        self.stroke_style.color = Color::TRANSPARENT;
        self
    }

    /// Sets the fill style of the brush.
    ///
    /// # Arguments
    ///
    /// * `fill_style` - The new fill style (color and other properties).
    pub fn fill_style(mut self, fill_style: FillStyle) -> Self {
        self.fill_style = fill_style;
        self
    }

    /// Gets the current stroke color of the brush.
    pub fn get_stroke_color(&self) -> Color {
        self.fill_style.color
    }

    /// Sets the stroke color of the brush.
    ///
    /// # Arguments
    ///
    /// * `color` - The new stroke color to be applied.
    pub fn stroke_color(mut self, color: Color) -> Self {
        self.stroke_style.color = color;
        self
    }

    /// Sets the stroke width (line width) of the brush.
    ///
    /// # Arguments
    ///
    /// * `stroke_width` - The new stroke width to be applied.
    pub fn line_width(mut self, line_width: u32) -> Self {
        self.stroke_style.line_width = line_width;
        self
    }

    /// Sets the stroke style of the brush.
    ///
    /// # Arguments
    ///
    /// * `stroke_style` - The new stroke style (color, width, and other properties).
    pub fn stroke_style(mut self, stroke_style: StrokeStyle) -> Self {
        self.stroke_style = stroke_style;
        self
    }

    /// Sets the stroke line join style for the brush.
    ///
    /// # Arguments
    ///
    /// * `line_join` - The line join style (e.g., miter, round, bevel).
    pub fn line_join(mut self, line_join: LineJoin) -> Self {
        self.stroke_style.line_join = line_join;
        self
    }

    /// Sets the stroke line cap style for the brush.
    ///
    /// # Arguments
    ///
    /// * `line_cap` - The line cap style (e.g., butt, round, square).
    pub fn line_cap(mut self, line_cap: LineCap) -> Self {
        self.stroke_style.line_cap = line_cap;
        self
    }

    /// Resets the brush to its default state.
    pub fn reset(self) -> Self {
        Self::default()
    }

    /// Checks if there is nothing to draw with the brush (i.e., both the fill and stroke colors are transparent).
    pub fn nothing_to_draw(&self) -> bool {
        self.fill_style.color.is_transparent() && self.stroke_style.color.is_transparent()
    }

    pub fn some<T>(self, opt: Option<T>, consequent: impl FnOnce(Self, T) -> Self) -> Self {
        if let Some(v) = opt {
            consequent(self, v)
        } else {
            self
        }
    }

    pub fn when<F>(self, cond: bool, consequent: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        if cond {
            consequent(self)
        } else {
            self
        }
    }

    pub fn when_or<C, A>(self, cond: bool, consequent: C, alternate: A) -> Self
    where
        C: FnOnce(Self) -> Self,
        A: FnOnce(Self) -> Self,
    {
        if cond {
            consequent(self)
        } else {
            alternate(self)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FillStyle {
    pub color: Color,
}

impl Default for FillStyle {
    fn default() -> Self {
        Self {
            color: Color::TRANSPARENT,
        }
    }
}

impl FillStyle {
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineJoin {
    Miter,
    Bevel,
    Round,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineCap {
    Round,
    Square,
    Butt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StrokeStyle {
    pub color: Color,
    pub line_width: u32,
    pub line_join: LineJoin,
    pub line_cap: LineCap,
    pub allow_overlap: bool,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            line_width: 2,
            line_join: LineJoin::Miter,
            line_cap: LineCap::Butt,
            allow_overlap: false,
        }
    }
}

impl StrokeStyle {
    pub fn allow_overlap(mut self, allow: bool) -> Self {
        self.allow_overlap = allow;
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    pub fn line_width(mut self, line_width: u32) -> Self {
        self.line_width = line_width;
        self
    }

    pub fn line_join(mut self, line_join: LineJoin) -> Self {
        self.line_join = line_join;
        self
    }

    pub fn line_cap(mut self, line_cap: LineCap) -> Self {
        self.line_cap = line_cap;
        self
    }

    pub fn default_join(mut self) -> Self {
        self.line_join = LineJoin::Miter;
        self
    }

    pub fn miter_join(mut self) -> Self {
        self.line_join = LineJoin::Miter;
        self
    }

    pub fn bevel_join(mut self) -> Self {
        self.line_join = LineJoin::Bevel;
        self
    }

    pub fn round_join(mut self) -> Self {
        self.line_join = LineJoin::Round;
        self
    }

    pub fn round_cap(mut self) -> Self {
        self.line_cap = LineCap::Round;
        self
    }

    /// aka with_butt_join lol
    pub fn default_cap(mut self) -> Self {
        self.line_cap = LineCap::Butt;
        self
    }

    pub fn square_cap(mut self) -> Self {
        self.line_cap = LineCap::Square;
        self
    }
}

#[derive(Debug, Clone)]
pub struct PathBrush {
    default: Brush,
    overrides: ahash::HashMap<Contour, Brush>,
}

impl PathBrush {
    pub fn new(default: Brush) -> Self {
        Self {
            default: default.clone(),
            ..Default::default()
        }
    }

    #[inline]
    pub fn set(&mut self, contour: Contour, brush: Brush) {
        self.overrides.insert(contour, brush);
    }

    #[inline]
    pub fn set_default(&mut self, default: Brush) {
        self.default = default;
    }

    #[inline]
    pub fn get_or_default(&self, contour: &Contour) -> Brush {
        self.overrides
            .get(contour)
            .cloned()
            .unwrap_or(self.default.clone())
    }
}

impl Default for PathBrush {
    fn default() -> Self {
        Self {
            default: Brush::filled(Color::WHITE),
            overrides: Default::default(),
        }
    }
}

impl From<Brush> for PathBrush {
    fn from(brush: Brush) -> Self {
        Self {
            default: brush,
            ..Default::default()
        }
    }
}

impl From<&Brush> for PathBrush {
    fn from(brush: &Brush) -> Self {
        Self {
            default: brush.clone(),
            ..Default::default()
        }
    }
}

impl<T> From<T> for PathBrush
where
    T: IntoIterator<Item = (Contour, Brush)>,
{
    fn from(value: T) -> Self {
        Self {
            default: Default::default(),
            overrides: value.into_iter().collect(),
        }
    }
}

pub struct PathBuilderWithBrush {
    path: PathBuilder,
    brush: PathBrush,
}

impl PathBuilderWithBrush {
    pub fn with_default_brush(&mut self, brush: Brush) {
        self.brush.default = brush;
    }

    #[inline]
    pub fn close(&mut self, brush: Brush) {
        self.brush.set(self.path.end(true), brush);
    }

    #[inline]
    pub fn end(&mut self, close: bool, brush: Brush) {
        self.brush.set(self.path.end(close), brush);
    }

    #[inline]
    pub fn polygon(&mut self, polygon: Polygon<Point>, brush: Brush) {
        self.brush.set(self.path.polygon(polygon), brush);
    }

    #[inline]
    pub fn circle(&mut self, center: Point, radius: f32, brush: Brush) {
        self.brush.set(self.path.circle(center, radius), brush);
    }

    #[inline]
    pub fn rect(&mut self, rect: &Rect<f32>, brush: Brush) {
        self.brush.set(self.path.rect(rect), brush);
    }

    #[inline]
    pub fn round_rect(&mut self, rect: &Rect<f32>, corners: &Corners<f32>, brush: Brush) {
        self.brush.set(self.path.round_rect(rect, corners), brush);
    }

    #[inline]
    pub fn draw(self, canvas: &mut Canvas) {
        canvas.draw_path(self.path, self.brush)
    }

    pub fn split(self) -> (PathBuilder, PathBrush) {
        (self.path, self.brush)
    }
}

impl Deref for PathBuilderWithBrush {
    type Target = PathBuilder;

    fn deref(&self) -> &Self::Target {
        &self.path
    }
}

impl DerefMut for PathBuilderWithBrush {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.path
    }
}

mod sealed {
    pub trait Sealed {}
}

pub trait PathBuilderBrushExt: sealed::Sealed {
    fn with_brush(self) -> PathBuilderWithBrush;
}

impl sealed::Sealed for PathBuilder {}

impl PathBuilderBrushExt for PathBuilder {
    fn with_brush(self) -> PathBuilderWithBrush {
        PathBuilderWithBrush {
            path: self,
            brush: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use ara_math::vec2;

    use crate::{
        path::{PathBuilder, PathEventsIter, PathGeometryBuilder, Point},
        Color,
    };

    use super::{Brush, PathBrush};

    #[test]
    fn paint_brush_with_path() {
        let mut path = PathBuilder::default();

        let mut brush = PathBrush::default();

        let leg_paint = Brush::filled(Color::RED).line_width(10);

        path.begin(vec2(0.0, 0.0));
        path.line_to(vec2(-20.0, 100.0));
        let leg_l = path.end(false);
        brush.set(leg_l, leg_paint.clone());

        path.begin(vec2(0.0, 0.0));
        path.line_to(vec2(20.0, 100.0));
        let leg_r = path.end(false);
        brush.set(leg_r, leg_paint.clone());

        let head_paint = Brush::filled(Color::WHITE);
        let head = path.circle(vec2(0.0, 0.0), 10.0);
        brush.set(head, head_paint.clone());

        let mut output = <Vec<Point>>::new();

        let mut builder =
            <PathGeometryBuilder<PathEventsIter>>::new(path.path_events(), &mut output)
                .map(|v| v.0);

        let leg_l_build = builder.next().expect("no contour");
        assert_eq!(leg_l, leg_l_build);

        let leg_r_build = builder.next().expect("no contour");
        assert_eq!(leg_r, leg_r_build);

        let head_build = builder.next().expect("no contour");
        assert_eq!(head, head_build);

        assert_eq!(builder.next(), None);

        assert_eq!(brush.get_or_default(&leg_l_build), leg_paint.clone());
        assert_eq!(brush.get_or_default(&leg_r_build), leg_paint);
        assert_eq!(brush.get_or_default(&head_build), head_paint);
    }
}
