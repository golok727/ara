use std::{
    fmt::Display,
    ops::{self, Add, Mul, Sub},
};

use crate::traits::{Half, IsZero, One, Zero};

#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vec2<T> {
    #[inline]
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl<T> Display for Vec2<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{ x: {}, y: {} }}", &self.x, &self.y)
    }
}

impl<T> Vec2<T>
where
    T: Clone + PartialOrd,
{
    pub fn max(&self, other: &Self) -> Self {
        Self {
            x: if self.x > other.x {
                self.x.clone()
            } else {
                other.x.clone()
            },
            y: if self.y > other.y {
                self.y.clone()
            } else {
                other.y.clone()
            },
        }
    }

    pub fn min(&self, other: &Self) -> Self {
        Self {
            x: if self.x <= other.x {
                self.x.clone()
            } else {
                other.x.clone()
            },
            y: if self.y <= other.y {
                self.y.clone()
            } else {
                other.y.clone()
            },
        }
    }

    pub fn clamp(&self, min: &Self, max: &Self) -> Self {
        self.max(min).min(max)
    }
}

impl<T> Vec2<T>
where
    T: Clone + Add<T, Output = T> + ops::Neg<Output = T>,
{
    pub fn normal(&self) -> Self {
        Vec2 {
            x: -self.y.clone(),
            y: self.x.clone(),
        }
    }
    pub fn rot90(self) -> Self {
        vec2(self.y, -self.x)
    }
}

impl<T> ops::Neg for Vec2<T>
where
    T: Clone + Add<T, Output = T> + ops::Neg<Output = T>,
{
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl<T> Vec2<T>
where
    T: Clone + Add<T, Output = T> + Mul<T, Output = T>,
{
    pub fn dot(&self, other: &Self) -> T {
        self.x.clone() * other.x.clone() + self.y.clone() * other.y.clone()
    }
}

impl<T> Vec2<T>
where
    T: Clone + Add<T, Output = T> + Mul<T, Output = T> + Sub<T, Output = T>,
{
    pub fn cross(&self, other: &Self) -> T {
        self.x.clone() * other.y.clone() - self.y.clone() * other.x.clone()
    }
}

impl From<Vec2<f32>> for [f32; 4] {
    fn from(Vec2 { x, y }: Vec2<f32>) -> Self {
        [x, y, 1.0, 1.0]
    }
}

impl<T> From<Vec2<T>> for (T, T) {
    fn from(Vec2 { x, y }: Vec2<T>) -> Self {
        (x, y)
    }
}

impl<T> From<[T; 2]> for Vec2<T>
where
    T: Clone,
{
    fn from(arr: [T; 2]) -> Self {
        Self {
            x: arr[0].clone(),
            y: arr[1].clone(),
        }
    }
}

impl<T> From<(T, T)> for Vec2<T> {
    fn from((x, y): (T, T)) -> Self {
        Self { x, y }
    }
}

impl From<Vec2<f32>> for [f32; 2] {
    fn from(Vec2 { x, y }: Vec2<f32>) -> Self {
        [x, y]
    }
}

impl<T> Half for Vec2<T>
where
    T: Half,
{
    fn half(&self) -> Self {
        Self {
            x: self.x.half(),
            y: self.y.half(),
        }
    }
}

impl<T> IsZero for Vec2<T>
where
    T: IsZero,
{
    fn is_zero(&self) -> bool {
        self.x.is_zero() && self.y.is_zero()
    }
}

impl<T> Zero for Vec2<T>
where
    T: Zero,
{
    fn zero() -> Self {
        Self {
            x: T::zero(),
            y: T::zero(),
        }
    }
}

impl<T> One for Vec2<T>
where
    T: One,
{
    fn one() -> Self {
        Self {
            x: T::one(),
            y: T::one(),
        }
    }
}

pub trait OneVec<T> {
    fn one() -> Vec2<T>;
}

macro_rules! impl_vec2_float {
    ($float:ty) => {
        impl Vec2<$float> {
            pub fn ceil(&self) -> Self {
                Self {
                    x: self.x.ceil(),
                    y: self.y.ceil(),
                }
            }

            pub fn floor(&self) -> Self {
                Self {
                    x: self.x.floor(),
                    y: self.y.floor(),
                }
            }

            pub fn round(&self) -> Self {
                Self {
                    x: self.x.round(),
                    y: self.y.round(),
                }
            }
            pub fn magnitude_sq(&self) -> $float {
                self.x * self.x + self.y * self.y
            }

            pub fn magnitude(&self) -> $float {
                (self.x * self.x + self.y * self.y).sqrt()
            }

            pub fn normalize(&self) -> Self {
                let len = (self.x * self.x + self.y * self.y).sqrt();
                if len == 0.0 {
                    Self::zero()
                } else {
                    Self {
                        x: self.x / len,
                        y: self.y / len,
                    }
                }
            }

            pub fn direction(&self, other: Self) -> Self {
                (*self - other).normalize()
            }

            pub fn angle(&self, v2: &Self) -> $float {
                let dot = self.x * v2.x + self.y * v2.y;
                let det = self.x * v2.y - self.y * v2.x;
                det.atan2(dot).abs()
            }
        }
    };
}

impl_vec2_float!(f32);
impl_vec2_float!(f64);

impl<T> Vec2<T> {
    pub fn map<U>(&self, f: impl Fn(&T) -> U) -> Vec2<U> {
        Vec2 {
            x: f(&self.x),
            y: f(&self.y),
        }
    }
}

impl<T> Vec2<T>
where
    T: Clone,
{
    pub fn map_cloned<U>(&self, f: impl Fn(T) -> U) -> Vec2<U> {
        Vec2 {
            x: f(self.x.clone()),
            y: f(self.y.clone()),
        }
    }
}

// Vector
impl<T> Add for Vec2<T>
where
    T: Add<T, Output = T>,
{
    type Output = Vec2<T>;

    fn add(self, rhs: Vec2<T>) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

// Scalar
impl<T> Add<T> for Vec2<T>
where
    T: Clone + Add<T, Output = T>,
{
    type Output = Vec2<T>;

    fn add(self, scalar: T) -> Self::Output {
        Vec2 {
            x: self.x + scalar.clone(),
            y: self.y + scalar.clone(),
        }
    }
}

// Begin Vector assign
impl<T> ops::AddAssign for Vec2<T>
where
    T: Add<T, Output = T> + Clone,
{
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs;
    }
}

impl<T> ops::SubAssign for Vec2<T>
where
    T: Sub<T, Output = T> + Clone,
{
    fn sub_assign(&mut self, rhs: Self) {
        *self = self.clone() - rhs;
    }
}

impl<T> ops::MulAssign for Vec2<T>
where
    T: Mul<T, Output = T> + Clone,
{
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.clone() * rhs;
    }
}

impl<T> ops::DivAssign for Vec2<T>
where
    T: ops::Div<T, Output = T> + Clone,
{
    fn div_assign(&mut self, rhs: Self) {
        *self = self.clone() / rhs;
    }
}
// END Vector assign

// Scalar
impl<T> Sub<T> for Vec2<T>
where
    T: Clone + Sub<T, Output = T>,
{
    type Output = Vec2<T>;

    fn sub(self, scalar: T) -> Self::Output {
        Vec2 {
            x: self.x - scalar.clone(),
            y: self.y - scalar.clone(),
        }
    }
}

// Vector
impl<T> Sub for Vec2<T>
where
    T: Sub<T, Output = T>,
{
    type Output = Vec2<T>;

    fn sub(self, rhs: Vec2<T>) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

// Vector
impl<T> Mul for Vec2<T>
where
    T: Mul<T, Output = T>,
{
    type Output = Vec2<T>;

    fn mul(self, rhs: Vec2<T>) -> Self::Output {
        Vec2 {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

// Scalar
impl<T> Mul<T> for Vec2<T>
where
    T: Clone + Mul<T, Output = T>,
{
    type Output = Vec2<T>;

    fn mul(self, scalar: T) -> Self::Output {
        Vec2 {
            x: self.x * scalar.clone(),
            y: self.y * scalar.clone(),
        }
    }
}

// Vector
impl<T> ops::Div for Vec2<T>
where
    T: ops::Div<T, Output = T>,
{
    type Output = Vec2<T>;

    fn div(self, rhs: Vec2<T>) -> Self::Output {
        Vec2 {
            x: self.x / rhs.x,
            y: self.y / rhs.y,
        }
    }
}

// Scalar
impl<T> ops::Div<T> for Vec2<T>
where
    T: Clone + ops::Div<T, Output = T>,
{
    type Output = Vec2<T>;

    fn div(self, scalar: T) -> Self::Output {
        Vec2 {
            x: self.x / scalar.clone(),
            y: self.y / scalar.clone(),
        }
    }
}

#[inline(always)]
pub fn vec2<T>(x: T, y: T) -> Vec2<T> {
    Vec2 { x, y }
}
