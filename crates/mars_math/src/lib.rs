#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Size<T = u32> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> {
    pub const fn new(width: T, height: T) -> Self {
        Self { width, height }
    }
}

impl Size<u32> {
    pub const fn area(&self) -> u32 {
        self.width * self.height
    }

    pub fn min(&self, other: Self) -> Self {
        size(self.width.min(other.width), self.height.min(other.height))
    }

    pub fn max(&self, other: Self) -> Self {
        size(self.width.max(other.width), self.height.max(other.height))
    }

    pub fn clamp(&self, min: Self, max: Self) -> Self {
        size(
            self.width.clamp(min.width, max.width),
            self.height.clamp(min.height, max.height),
        )
    }
}

pub const fn size<T>(width: T, height: T) -> Size<T> {
    Size::new(width, height)
}

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

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Position<T = i32> {
    pub x: T,
    pub y: T,
}

impl<T> Position<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

impl Position<i32> {
    pub const ZERO: Self = Self::new(0, 0);

    pub const fn delta(self, other: Self) -> Delta<i32> {
        Delta::new(other.x - self.x, other.y - self.y)
    }
}

pub const fn pos<T>(x: T, y: T) -> Position<T> {
    Position::new(x, y)
}

impl std::ops::Add for Position<i32> {
    type Output = Self;
    #[track_caller]
    fn add(self, rhs: Self) -> Self::Output {
        pos(self.x + rhs.x, self.y + rhs.y)
    }
}
impl std::ops::Sub for Position<i32> {
    type Output = Self;
    #[track_caller]
    fn sub(self, rhs: Self) -> Self::Output {
        pos(self.x - rhs.x, self.y - rhs.y)
    }
}
impl std::ops::Mul for Position<i32> {
    type Output = Self;
    #[track_caller]
    fn mul(self, rhs: Self) -> Self::Output {
        pos(self.x * rhs.x, self.y * rhs.y)
    }
}
impl std::ops::Div for Position<i32> {
    type Output = Self;
    #[track_caller]
    fn div(self, rhs: Self) -> Self::Output {
        pos(self.x / rhs.x, self.y / rhs.y)
    }
}

impl std::ops::Add<i32> for Position<i32> {
    type Output = Self;
    #[track_caller]
    fn add(self, rhs: i32) -> Self::Output {
        pos(self.x + rhs, self.y + rhs)
    }
}
impl std::ops::Sub<i32> for Position<i32> {
    type Output = Self;
    #[track_caller]
    fn sub(self, rhs: i32) -> Self::Output {
        pos(self.x - rhs, self.y - rhs)
    }
}
impl std::ops::Mul<i32> for Position<i32> {
    type Output = Self;
    #[track_caller]
    fn mul(self, rhs: i32) -> Self::Output {
        pos(self.x * rhs, self.y * rhs)
    }
}
impl std::ops::Div<i32> for Position<i32> {
    type Output = Self;
    #[track_caller]
    fn div(self, rhs: i32) -> Self::Output {
        pos(self.x / rhs, self.y / rhs)
    }
}

impl std::ops::AddAssign for Position<i32> {
    #[track_caller]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}
impl std::ops::SubAssign for Position<i32> {
    #[track_caller]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}
impl std::ops::MulAssign for Position<i32> {
    #[track_caller]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}
impl std::ops::DivAssign for Position<i32> {
    #[track_caller]
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs
    }
}
impl std::ops::AddAssign<i32> for Position<i32> {
    #[track_caller]
    fn add_assign(&mut self, rhs: i32) {
        *self = *self + rhs
    }
}
impl std::ops::SubAssign<i32> for Position<i32> {
    #[track_caller]
    fn sub_assign(&mut self, rhs: i32) {
        *self = *self - rhs
    }
}
impl std::ops::MulAssign<i32> for Position<i32> {
    #[track_caller]
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs
    }
}
impl std::ops::DivAssign<i32> for Position<i32> {
    #[track_caller]
    fn div_assign(&mut self, rhs: i32) {
        *self = *self / rhs
    }
}

impl std::ops::Not for Position<i32> {
    type Output = Self;
    fn not(self) -> Self::Output {
        pos(-self.x, -self.y)
    }
}
