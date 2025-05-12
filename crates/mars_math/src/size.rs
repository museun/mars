use crate::{Anchor2, Num, Position};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, Hash)]
pub struct Size<T: Num = u32> {
    pub width: T,
    pub height: T,
}

impl<T: Num> Size<T> {
    pub const ZERO: Self = Self::new(T::ZERO, T::ZERO);

    #[must_use]
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub fn area(&self) -> T {
        self.width * self.height
    }

    #[must_use]
    pub fn min(&self, other: Self) -> Self {
        Self::new(self.width.min(other.width), self.height.min(other.height))
    }

    #[must_use]
    pub fn max(&self, other: Self) -> Self {
        Self::new(self.width.max(other.width), self.height.max(other.height))
    }

    #[must_use]
    pub fn clamp(&self, min: Self, max: Self) -> Self {
        Self::new(
            self.width.clamp(min.width, max.width),
            self.height.clamp(min.height, max.height),
        )
    }

    #[must_use]
    pub fn to_position(&self) -> Position<T> {
        Position::new(self.width, self.height)
    }

    #[must_use]
    pub fn to_float(&self) -> Size<f64> {
        Size::new(self.width.into(), self.height.into())
    }

    // NOTE: this isn't a vector. its a size. we'll have Vec2 / Vec3 for that kind of math
}

impl Size<f64> {
    #[must_use]
    #[track_caller]
    pub fn to_signed(&self) -> Size<i32> {
        self.to_signed_checked().unwrap()
    }

    #[must_use]
    pub fn to_signed_checked(&self) -> Option<Size<i32>> {
        let (w, h) = (self.width.round_ties_even(), self.height.round_ties_even());
        let w = i32::try_from(w as isize).ok()?;
        let h = i32::try_from(h as isize).ok()?;
        Some(Size::new(w, h))
    }
}

impl Size<i32> {
    #[must_use]
    #[track_caller]
    pub fn to_unsigned(&self) -> Size<u32> {
        self.to_unsigned_checked().unwrap()
    }

    #[must_use]
    pub fn to_unsigned_checked(&self) -> Option<Size<u32>> {
        let w = u32::try_from(self.width).ok()?;
        let h = u32::try_from(self.height).ok()?;
        Some(Size::new(w, h))
    }
}

impl Size<u32> {
    #[must_use]
    #[track_caller]
    pub fn to_signed(&self) -> Size<i32> {
        self.to_signed_checked().unwrap()
    }

    #[must_use]
    pub fn to_signed_checked(&self) -> Option<Size<i32>> {
        let (w, h) = (self.width, self.height);
        let w = i32::try_from(w).ok()?;
        let h = i32::try_from(h).ok()?;
        Some(Size::new(w, h))
    }
}

crate::macros::ops_impl! {
    Size { width, height } => i32
    Size { width, height } => u32
    Size { width, height } => f32
    Size { width, height } => f64
}

impl std::ops::Mul<Anchor2> for Size {
    type Output = Size<f64>;
    #[track_caller]
    fn mul(self, rhs: Anchor2) -> Self::Output {
        let (x, y) = rhs.factor();
        let this = self.to_float();
        Size::new(this.width * x, this.height * y)
    }
}

impl std::ops::Div<Anchor2> for Size {
    type Output = Size<f64>;
    #[track_caller]
    fn div(self, rhs: Anchor2) -> Self::Output {
        let (x, y) = rhs.factor();
        let this = self.to_float();
        Size::new(this.width / x, this.height / y)
    }
}
