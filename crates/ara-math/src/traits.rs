pub trait IsZero {
    fn is_zero(&self) -> bool;
}

pub trait One
where
    Self: Sized,
{
    fn one() -> Self;
    fn to_one(&mut self) {
        *self = Self::one()
    }
}

pub trait Zero
where
    Self: Sized,
{
    fn zero() -> Self;
    fn to_zero(&mut self) {
        *self = Self::zero()
    }
}

pub trait Half {
    fn half(&self) -> Self;
}

macro_rules! impl_is_zero {
    ($type:ty, $cmp:tt) => {
        impl IsZero for $type {
            #[inline]
            fn is_zero(&self) -> bool {
                *self == $cmp
            }
        }
    };
}

impl_is_zero!(f64, 0.0);
impl_is_zero!(f32, 0.0);

impl_is_zero!(i32, 0);
impl_is_zero!(i16, 0);
impl_is_zero!(i8, 0);

impl_is_zero!(u32, 0);
impl_is_zero!(u16, 0);
impl_is_zero!(u8, 0);
impl_is_zero!(usize, 0);

macro_rules! impl_half {
    ($type:ty, $div:tt) => {
        impl Half for $type {
            #[inline]
            fn half(&self) -> Self {
                *self / $div
            }
        }
    };
}

impl_half!(f64, 2.0);
impl_half!(f32, 2.0);
impl_half!(i32, 2);
impl_half!(i16, 2);
impl_half!(i8, 2);

impl_half!(u32, 2);
impl_half!(u16, 2);
impl_half!(u8, 2);
impl_half!(usize, 2);

macro_rules! impl_zero_and_one {
    ($type:ty, $zero:tt, $one:tt) => {
        impl Zero for $type {
            #[inline]
            fn zero() -> Self {
                $zero
            }
        }

        impl One for $type {
            #[inline]
            fn one() -> Self {
                $one
            }
        }
    };
}

impl_zero_and_one!(f64, 0.0, 1.0);
impl_zero_and_one!(f32, 0.0, 1.0);

impl_zero_and_one!(i32, 0, 1);
impl_zero_and_one!(i16, 0, 1);
impl_zero_and_one!(i8, 0, 1);

impl_zero_and_one!(u32, 0, 1);
impl_zero_and_one!(u16, 0, 1);
impl_zero_and_one!(u8, 0, 1);
impl_zero_and_one!(usize, 0, 1);
