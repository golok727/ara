use std::{
    fmt::{Debug, Display},
    ops::{Add, Div, Mul},
};

use crate::traits::{Half, IsZero, Zero};

use super::Vec2;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}
impl<T> std::fmt::Display for Size<T>
where
    T: Display + Clone + Debug + Default,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ width: {}, height: {} }}", &self.width, &self.height)
    }
}

impl<T> Size<T>
where
    T: Half,
{
    pub fn center(&self) -> Vec2<T> {
        Vec2 {
            x: self.width.half(),
            y: self.height.half(),
        }
    }
}

impl<T> Half for Size<T>
where
    T: Half,
{
    fn half(&self) -> Self {
        Self {
            width: self.width.half(),
            height: self.height.half(),
        }
    }
}

impl<T> From<&Size<T>> for Vec2<T>
where
    T: Clone,
{
    fn from(value: &Size<T>) -> Self {
        Self {
            x: value.width.clone(),
            y: value.height.clone(),
        }
    }
}

impl<T> From<Size<T>> for Vec2<T> {
    fn from(value: Size<T>) -> Self {
        Self {
            x: value.width,
            y: value.height,
        }
    }
}

impl<T> IsZero for Size<T>
where
    T: IsZero,
{
    fn is_zero(&self) -> bool {
        self.width.is_zero() || self.height.is_zero()
    }
}

impl<T> Zero for Size<T>
where
    T: Zero,
{
    fn zero() -> Self {
        Self {
            width: T::zero(),
            height: T::zero(),
        }
    }
}

impl<T> Size<T>
where
    T: IsZero,
{
    pub fn empty(&self) -> bool {
        self.width.is_zero() && self.height.is_zero()
    }
}

impl<T> Size<T> {
    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> Size<U> {
        Size {
            width: f(&self.width),
            height: f(&self.height),
        }
    }
}

impl<T> Size<T>
where
    T: Clone,
{
    pub fn map_cloned<U>(&self, f: impl Fn(T) -> U) -> Size<U> {
        Size {
            width: f(self.width.clone()),
            height: f(self.height.clone()),
        }
    }
}

impl<T> Add<Size<T>> for Size<T>
where
    T: Clone + Add<T, Output = T>,
{
    type Output = Size<T>;

    fn add(self, rhs: Size<T>) -> Self::Output {
        Self {
            width: self.width.clone() + rhs.width.clone(),
            height: self.height.clone() + rhs.height.clone(),
        }
    }
}

impl<T> Add<Vec2<T>> for Size<T>
where
    T: Clone + Add<T, Output = T>,
{
    type Output = Size<T>;

    fn add(self, rhs: Vec2<T>) -> Self::Output {
        Self {
            width: self.width.clone() + rhs.x.clone(),
            height: self.height.clone() + rhs.y.clone(),
        }
    }
}

impl<T> Add<Size<T>> for Vec2<T>
where
    T: Clone + Add<T, Output = T>,
{
    type Output = Vec2<T>;

    fn add(self, rhs: Size<T>) -> Self::Output {
        Self {
            x: self.x.clone() + rhs.width.clone(),
            y: self.y.clone() + rhs.height.clone(),
        }
    }
}

// add a scalar value to this size
impl<T> Add<T> for Size<T>
where
    T: Clone + Add<T, Output = T>,
{
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self {
            width: self.width.clone() + rhs.clone(),
            height: self.height.clone() + rhs.clone(),
        }
    }
}

impl<T> Add<T> for &Size<T>
where
    T: Clone + Add<T, Output = T>,
{
    type Output = Size<T>;

    fn add(self, rhs: T) -> Self::Output {
        Size {
            width: self.width.clone() + rhs.clone(),
            height: self.height.clone() + rhs.clone(),
        }
    }
}

// size / scalar
impl<T> Div<T> for Size<T>
where
    T: Clone + Div<T, Output = T>,
{
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self {
            width: self.width.clone() / rhs.clone(),
            height: self.height.clone() / rhs.clone(),
        }
    }
}

impl<T> Size<T>
where
    T: Mul<T, Output = T> + Clone,
{
    pub fn scale(&self, scalar: T) -> Self {
        Self {
            width: self.width.clone() * scalar.clone(),
            height: self.height.clone() * scalar.clone(),
        }
    }

    pub fn scale_wh(&self, sw: T, sh: T) -> Self {
        Self {
            width: self.width.clone() * sw.clone(),
            height: self.height.clone() * sh.clone(),
        }
    }
}

impl<T> Size<T>
where
    T: PartialOrd + Clone,
{
    pub fn max(&self, other: &Self) -> Self {
        Size {
            width: if self.width >= other.width {
                self.width.clone()
            } else {
                other.width.clone()
            },
            height: if self.height >= other.height {
                self.height.clone()
            } else {
                other.height.clone()
            },
        }
    }

    pub fn min(&self, other: &Self) -> Self {
        Size {
            width: if self.width >= other.width {
                other.width.clone()
            } else {
                self.width.clone()
            },
            height: if self.height >= other.height {
                other.height.clone()
            } else {
                self.height.clone()
            },
        }
    }

    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        self.max(min).min(max)
    }
}
