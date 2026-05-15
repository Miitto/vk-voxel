use std::ops::*;

pub trait Number:
    Add<Self, Output = Self>
    + AddAssign
    + Sub<Self, Output = Self>
    + SubAssign
    + Mul<Self, Output = Self>
    + MulAssign
    + Div<Self, Output = Self>
    + DivAssign
    + Sized
    + Copy
    + Default
{
}

pub trait SquareRoot: Number {
    fn sqrt(self) -> Self;
}

pub trait Integer: Number + Rem {}
pub trait SignedInteger: Integer + Neg {}
pub trait Float: Number {}

impl Number for u8 {}
impl Integer for u8 {}
impl Number for u16 {}
impl Integer for u16 {}
impl Number for u32 {}
impl Integer for u32 {}
impl Number for u64 {}
impl Integer for u64 {}
impl Number for u128 {}
impl Integer for u128 {}
impl Number for usize {}
impl Integer for usize {}

impl Number for i8 {}
impl Integer for i8 {}
impl SignedInteger for i8 {}
impl Number for i16 {}
impl Integer for i16 {}
impl SignedInteger for i16 {}
impl Number for i32 {}
impl Integer for i32 {}
impl SignedInteger for i32 {}
impl Number for i64 {}
impl Integer for i64 {}
impl SignedInteger for i64 {}
impl Number for i128 {}
impl Integer for i128 {}
impl SignedInteger for i128 {}
impl Number for isize {}
impl Integer for isize {}
impl SignedInteger for isize {}

impl Number for f32 {}
impl Float for f32 {}
impl SquareRoot for f32 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}
impl Number for f64 {}
impl Float for f64 {}
impl SquareRoot for f64 {
    fn sqrt(self) -> Self {
        self.sqrt()
    }
}
