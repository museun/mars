#[derive(Copy, Clone, Default, Debug, PartialEq)]
pub enum Axis {
    #[default]
    Horizontal,
    Vertical,
}

impl Axis {
    #[must_use]
    pub fn main<T>(&self, value: impl Into<(T, T)>) -> T {
        let (x, y) = value.into();
        match self {
            Self::Horizontal => x,
            Self::Vertical => y,
        }
    }

    #[must_use]
    pub fn cross<T>(&self, value: impl Into<(T, T)>) -> T {
        let (x, y) = value.into();
        match self {
            Self::Horizontal => y,
            Self::Vertical => x,
        }
    }

    #[must_use]
    pub fn pack<T, R>(&self, main: T, cross: T) -> R
    where
        R: From<(T, T)>,
    {
        match self {
            Self::Horizontal => R::from((main, cross)),
            Self::Vertical => R::from((cross, main)),
        }
    }

    #[must_use]
    pub fn unpack<T>(&self, value: impl Into<(T, T)>) -> (T, T) {
        let (x, y) = value.into();
        match self {
            Self::Horizontal => (x, y),
            Self::Vertical => (y, x),
        }
    }

    #[must_use]
    pub const fn is_vertical(&self) -> bool {
        matches!(self, Self::Vertical)
    }

    #[must_use]
    pub const fn is_horizontal(&self) -> bool {
        matches!(self, Self::Horizontal)
    }
}

impl std::ops::Not for Axis {
    type Output = Self;
    fn not(self) -> Self::Output {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}
