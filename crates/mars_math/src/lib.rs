use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

mod macros;

mod size;
pub use size::Size;

mod delta;
pub use delta::Delta;

mod position;
pub use position::Position;

mod anchor;
pub use anchor::{Anchor, Anchor2};

mod axis;
pub use axis::Axis;

mod margin;
pub use margin::Margin;

pub trait Num
where
    Self: PartialEq
        + Copy
        + Add<Self, Output = Self>
        + AddAssign<Self>
        + Sub<Self, Output = Self>
        + SubAssign<Self>
        + Mul<Self, Output = Self>
        + MulAssign<Self>
        + Div<Self, Output = Self>
        + DivAssign<Self>
        + Into<f64>,
{
    const MIN: Self;
    const MAX: Self;
    const ZERO: Self;
    const ONE: Self;

    fn min(self, other: Self) -> Self;
    fn max(self, other: Self) -> Self;
    fn clamp(self, min: Self, max: Self) -> Self;
}

impl Num for i32 {
    const MIN: Self = i32::MIN;
    const MAX: Self = i32::MAX;
    const ZERO: Self = 0;
    const ONE: Self = 1;

    fn min(self, other: Self) -> Self {
        Ord::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        Ord::max(self, other)
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        Ord::clamp(self, min, max)
    }
}

impl Num for u32 {
    const MIN: Self = u32::MIN;
    const MAX: Self = u32::MAX;
    const ZERO: Self = 0;
    const ONE: Self = 1;

    fn min(self, other: Self) -> Self {
        Ord::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        Ord::max(self, other)
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        Ord::clamp(self, min, max)
    }
}

impl Num for f32 {
    const MIN: Self = f32::MIN;
    const MAX: Self = f32::MAX;
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

    fn min(self, other: Self) -> Self {
        f32::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        f32::max(self, other)
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        f32::clamp(self, min, max)
    }
}

impl Num for f64 {
    const MIN: Self = f64::MIN;
    const MAX: Self = f64::MAX;
    const ZERO: Self = 0.0;
    const ONE: Self = 1.0;

    fn min(self, other: Self) -> Self {
        f64::min(self, other)
    }

    fn max(self, other: Self) -> Self {
        f64::max(self, other)
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        f64::clamp(self, min, max)
    }
}
