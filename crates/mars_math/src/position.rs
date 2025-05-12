use crate::{Delta, Num, Size};

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Position<T: Num = i32> {
    pub x: T,
    pub y: T,
}

impl<T: Num> Position<T> {
    pub const ZERO: Self = Self::new(T::ZERO, T::ZERO);

    #[must_use]
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }

    #[must_use]
    pub fn min(&self, other: Self) -> Self {
        Self::new(self.x.min(other.x), self.y.min(other.y))
    }

    #[must_use]
    pub fn max(&self, other: Self) -> Self {
        Self::new(self.x.max(other.x), self.y.max(other.y))
    }

    #[must_use]
    pub fn clamp(&self, min: Self, max: Self) -> Self {
        Self::new(self.x.clamp(min.x, max.x), self.y.clamp(min.y, max.y))
    }

    #[must_use]
    pub fn to_size(&self) -> Size<T> {
        Size::new(self.x, self.y)
    }

    // distance
    // length
    // lerp
    // abs
}

impl Position<i32> {
    #[must_use]
    pub const fn delta(self, other: Self) -> Delta<i32> {
        Delta::new(other.x - self.x, other.y - self.y)
    }

    /// Panics: If the position is negative this'll panic
    #[must_use]
    #[track_caller]
    pub fn to_unsigned(self) -> Position<u32> {
        self.to_unsigned_checked().unwrap()
    }

    #[must_use]
    pub fn to_unsigned_checked(self) -> Option<Position<u32>> {
        let x = u32::try_from(self.x).ok()?;
        let y = u32::try_from(self.y).ok()?;
        Some(Position::new(x, y))
    }
}

impl Position<u32> {
    /// Panics: If the position is larger than i32::MAX
    #[must_use]
    #[track_caller]
    pub fn to_signed(self) -> Position<i32> {
        self.to_signed_checked().unwrap()
    }

    #[must_use]
    pub fn to_signed_checked(self) -> Option<Position<i32>> {
        let x = i32::try_from(self.x).ok()?;
        let y = i32::try_from(self.y).ok()?;
        Some(Position::new(x, y))
    }
}

super::macros::ops_impl! {
    Position { x, y } => i32
    Position { x, y } => u32
    Position { x, y } => f32
    Position { x, y } => f64
}

impl std::ops::Not for Position<i32> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl std::ops::Not for Position<f32> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}

impl std::ops::Not for Position<f64> {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self::new(-self.x, -self.y)
    }
}
