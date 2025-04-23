use core::f32;
use std::{
    cmp::PartialOrd,
    fmt::Debug,
    ops::{Add, Sub},
};

use crate::traits::{Half, IsZero, Zero};

use super::{Size, Vec2};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rect<T> {
    pub origin: Vec2<T>,
    pub size: Size<T>,
}

impl<T> From<(T, T, T, T)> for Rect<T> {
    fn from(v: (T, T, T, T)) -> Self {
        Self {
            origin: Vec2 { x: v.0, y: v.1 },
            size: Size {
                width: v.2,
                height: v.3,
            },
        }
    }
}

impl<T> Rect<T> {
    pub fn xywh(x: T, y: T, width: T, height: T) -> Self {
        Self {
            origin: Vec2 { x, y },
            size: Size { width, height },
        }
    }

    pub fn from_origin_size(origin: Vec2<T>, size: Size<T>) -> Self {
        Self { origin, size }
    }
}

impl<T: Clone> Rect<T> {
    #[inline(always)]
    pub fn x(&self) -> T {
        self.origin.x.clone()
    }

    #[inline(always)]
    pub fn y(&self) -> T {
        self.origin.y.clone()
    }

    #[inline(always)]
    pub fn width(&self) -> T {
        self.size.width.clone()
    }

    #[inline(always)]
    pub fn height(&self) -> T {
        self.size.height.clone()
    }

    #[inline(always)]
    pub fn position(&self) -> Vec2<T> {
        self.origin.clone()
    }

    #[inline(always)]
    pub fn size(&self) -> Size<T> {
        self.size.clone()
    }
}

impl<T> Zero for Rect<T>
where
    T: Zero,
{
    fn zero() -> Self {
        Self {
            origin: <Vec2<T>>::zero(),
            size: <Size<T>>::zero(),
        }
    }

    fn to_zero(&mut self) {
        self.size = Size::zero()
    }
}

impl<T> Half for Rect<T>
where
    T: Half + Clone,
{
    fn half(&self) -> Self {
        Self {
            origin: self.origin.clone(),
            size: self.size.half(),
        }
    }
}

impl<T> Rect<T>
where
    T: IsZero,
{
    /// returns if the width and height are zero
    pub fn empty(&self) -> bool {
        self.size.is_zero()
    }
}

impl<T> Rect<T>
where
    T: Clone + Sub<T, Output = T>,
{
    pub fn from_corners(upper_left: Vec2<T>, bottom_right: Vec2<T>) -> Self {
        let origin = Vec2 {
            x: upper_left.x.clone(),
            y: upper_left.y.clone(),
        };
        let size = Size {
            width: bottom_right.x - upper_left.x,
            height: bottom_right.y - upper_left.y,
        };
        Self { origin, size }
    }
}

impl<T> Rect<T>
where
    T: Clone + PartialOrd + Add<T, Output = T>,
{
    pub fn intersects(&self, other: &Self) -> bool {
        let my_lower_right = self.bottom_right();
        let their_lower_right = other.bottom_right();

        self.origin.x < their_lower_right.x
            && my_lower_right.x > other.origin.x
            && self.origin.y < their_lower_right.y
            && my_lower_right.y > other.origin.y
    }
}

impl<T> Rect<T>
where
    T: Clone + Add<T, Output = T> + Half,
{
    pub fn center(&self) -> Vec2<T> {
        Vec2 {
            x: self.origin.x.clone() + self.size.width.clone().half(),
            y: self.origin.y.clone() + self.size.height.clone().half(),
        }
    }
}

impl<T> Rect<T>
where
    T: Clone + Sub<T, Output = T> + Half,
{
    /// Make the rect cented at the origin
    pub fn centered(&self) -> Self {
        Self {
            origin: Vec2 {
                x: self.origin.x.clone() - self.size.width.clone().half(),
                y: self.origin.y.clone() - self.size.height.clone().half(),
            },
            size: self.size.clone(),
        }
    }
}

impl<T> Rect<T>
where
    T: Clone + PartialOrd + Add<T, Output = T> + Sub<T, Output = T>,
{
    pub fn intersect(&self, other: &Self) -> Self {
        let upper_left = self.origin.max(&other.origin);
        let bottom_right = self.bottom_right().min(&other.bottom_right());
        Self::from_corners(upper_left, bottom_right)
    }

    pub fn union(&self, other: &Self) -> Self {
        let top_left = self.origin.min(&other.origin);
        let bottom_right = self.bottom_right().max(&other.bottom_right());
        Self::from_corners(top_left, bottom_right)
    }
}

impl<T> Add<Vec2<T>> for Rect<T>
where
    T: Add<T, Output = T> + Clone,
{
    type Output = Self;

    fn add(self, rhs: Vec2<T>) -> Self {
        Self {
            origin: self.origin + rhs,
            size: self.size,
        }
    }
}

impl<T> Rect<T>
where
    T: Clone + Add<T, Output = T> + Sub<T, Output = T>,
{
    pub fn extend(&mut self, delta: T) -> Self {
        Self {
            origin: self.origin.clone() - delta.clone(),
            size: self.size.clone() + delta.clone(),
        }
    }

    pub fn pad(&mut self, padding: Size<T>) -> Self {
        Self {
            origin: self.origin.clone() - Vec2::new(padding.width.clone(), padding.height.clone()),
            size: self.size.clone() + padding,
        }
    }
}

impl<T> Rect<T>
where
    T: Add<T, Output = T> + PartialOrd + Clone,
{
    pub fn contains_point(&self, point: &Vec2<T>) -> bool {
        point.x >= self.origin.x
            && point.x <= self.origin.x.clone() + self.size.width.clone()
            && point.y >= self.origin.y
            && point.y <= self.origin.y.clone() + self.size.height.clone()
    }
}

impl<T> Rect<T>
// copy ?
where
    T: Add<T, Output = T> + PartialOrd + Clone,
{
    pub fn contains(&self, other: &Rect<T>) -> bool {
        let is_x_inside = other.origin.x.clone() >= self.origin.x.clone()
            && other.origin.x.clone() + other.size.width.clone()
                <= self.origin.x.clone() + self.size.width.clone();

        let is_y_inside = other.origin.y.clone() >= self.origin.y.clone()
            && other.origin.y.clone() + other.size.height.clone()
                <= self.origin.y.clone() + self.size.height.clone();

        is_x_inside && is_y_inside
    }
}

impl Rect<f32> {
    pub const EVERYTHING: Self = Self {
        origin: Vec2 {
            x: f32::MIN,
            y: f32::MIN,
        },
        size: Size {
            width: f32::INFINITY,
            height: f32::INFINITY,
        },
    };

    pub const NOTHING: Self = Self {
        origin: Vec2 {
            x: f32::INFINITY,
            y: f32::INFINITY,
        },
        size: Size {
            width: -f32::INFINITY,
            height: -f32::INFINITY,
        },
    };
}

impl<T> Rect<T>
where
    T: Clone + Add<T, Output = T>,
{
    pub fn min(&self) -> Vec2<T> {
        self.origin.clone()
    }

    pub fn max(&self) -> Vec2<T> {
        Vec2 {
            x: self.origin.x.clone() + self.size.width.clone(),
            y: self.origin.y.clone() + self.size.height.clone(),
        }
    }

    pub fn top_left(&self) -> Vec2<T> {
        self.origin.clone()
    }

    pub fn top_right(&self) -> Vec2<T> {
        Vec2 {
            x: self.origin.x.clone() + self.size.width.clone(),
            y: self.origin.y.clone(),
        }
    }

    pub fn bottom_left(&self) -> Vec2<T> {
        Vec2 {
            x: self.origin.x.clone(),
            y: self.origin.y.clone() + self.size.height.clone(),
        }
    }

    pub fn bottom_right(&self) -> Vec2<T> {
        Vec2 {
            x: self.origin.x.clone() + self.size.width.clone(),
            y: self.origin.y.clone() + self.size.height.clone(),
        }
    }
}
