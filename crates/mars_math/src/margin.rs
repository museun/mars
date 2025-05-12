use crate::{Num, Position, Size};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Margin<T: Num = u32> {
    pub left: T,
    pub right: T,
    pub top: T,
    pub bottom: T,
}

impl<T: Num> Margin<T> {
    pub const ZERO: Self = Self::same(T::ZERO);
    pub const ONE: Self = Self::same(T::ONE);

    #[must_use]
    pub const fn new(left: T, right: T, top: T, bottom: T) -> Self {
        Self {
            left,
            right,
            top,
            bottom,
        }
    }

    #[must_use]
    pub const fn symmetric(x: T, y: T) -> Self {
        Self {
            left: x,
            right: x,
            top: y,
            bottom: y,
        }
    }

    #[must_use]
    pub const fn same(margin: T) -> Self {
        Self::symmetric(margin, margin)
    }

    #[must_use]
    pub fn sum(&self) -> Size<T> {
        Size::new(self.left + self.right, self.top + self.bottom)
    }

    #[must_use]
    pub const fn left_top(&self) -> Position<T> {
        Position::new(self.left, self.top)
    }

    #[must_use]
    pub const fn right_bottom(&self) -> Position<T> {
        Position::new(self.right, self.bottom)
    }
}
