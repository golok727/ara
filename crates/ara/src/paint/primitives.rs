use crate::{math::Corners, path::Path, Zero};
use std::fmt::Debug;

use crate::math::{Rect, Vec2};

use super::PathBrush;

#[derive(Debug, Clone)]
pub enum Primitive {
    Quad(Quad),
    Path { path: Path, brush: PathBrush },
    Circle(Circle),
}

#[derive(Debug, Default, Clone)]
pub struct Circle {
    pub center: Vec2<f32>,
    pub radius: f32,
}

impl Circle {
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    pub fn pos(mut self, cx: f32, cy: f32) -> Self {
        self.center.x = cx;
        self.center.y = cy;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Quad {
    pub bounds: Rect<f32>,
    pub corners: Corners<f32>,
}

impl Quad {
    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.bounds.size.width = width;
        self.bounds.size.height = height;
        self
    }

    pub fn pos(mut self, x: f32, y: f32) -> Self {
        self.bounds.origin.x = x;
        self.bounds.origin.y = y;
        self
    }

    pub fn rect(mut self, rect: Rect<f32>) -> Self {
        self.bounds = rect;
        self
    }

    pub fn corners(mut self, corners: Corners<f32>) -> Self {
        self.corners = corners;
        self
    }
}

impl Default for Quad {
    fn default() -> Self {
        Self {
            bounds: Rect::zero(),
            corners: Corners::default(),
        }
    }
}

#[inline]
pub fn quad() -> Quad {
    Quad::default()
}

#[inline]
pub fn circle() -> Circle {
    Circle::default()
}

impl From<Quad> for Primitive {
    #[inline]
    fn from(quad: Quad) -> Self {
        Primitive::Quad(quad)
    }
}

impl From<Circle> for Primitive {
    #[inline]
    fn from(circle: Circle) -> Self {
        Primitive::Circle(circle)
    }
}
