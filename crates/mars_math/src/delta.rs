#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Delta<T = f32> {
    pub x: T,
    pub y: T,
}

impl<T> Delta<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Delta<f32> {
    pub const ZERO: Self = Self::new(0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 1.0);
}

impl Delta<i32> {
    pub const ZERO: Self = Self::new(0, 0);
    pub const ONE: Self = Self::new(1, 1);
}
