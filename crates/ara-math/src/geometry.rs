use std::fmt::Debug;

use crate::{traits::IsZero, Zero};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Corners<T>
where
    T: Clone + Default + Debug,
{
    pub top_left: T,
    pub top_right: T,
    pub bottom_left: T,
    pub bottom_right: T,
}

impl<T> Corners<T>
where
    T: Clone + Debug + Default,
{
    pub fn with_each(top_left: T, top_right: T, bottom_left: T, bottom_right: T) -> Self {
        Self {
            top_left,
            top_right,
            bottom_left,
            bottom_right,
        }
    }

    pub fn with_all(v: T) -> Self {
        Self {
            top_left: v.clone(),
            top_right: v.clone(),
            bottom_left: v.clone(),
            bottom_right: v,
        }
    }

    pub fn with_top_left(mut self, v: T) -> Self {
        self.top_left = v;
        self
    }

    pub fn with_top_right(mut self, v: T) -> Self {
        self.top_right = v;
        self
    }

    pub fn with_bottom_left(mut self, v: T) -> Self {
        self.bottom_left = v;
        self
    }

    pub fn with_bottom_right(mut self, v: T) -> Self {
        self.bottom_right = v;
        self
    }
}

impl<T> Corners<T>
where
    T: Clone + Debug + Default + PartialOrd + Ord,
{
    pub fn max(&self) -> T {
        self.top_left
            .clone()
            .max(self.top_right.clone())
            .max(self.bottom_right.clone())
            .max(self.bottom_left.clone())
    }
}

impl<T> Zero for Corners<T>
where
    T: Zero + Clone + Debug + Default,
{
    fn zero() -> Self {
        Self::with_all(T::zero())
    }
}

impl<T> IsZero for Corners<T>
where
    T: IsZero + Clone + Debug + Default,
{
    fn is_zero(&self) -> bool {
        self.top_left.is_zero()
            && self.top_right.is_zero()
            && self.bottom_left.is_zero()
            && self.bottom_right.is_zero()
    }
}
