macro_rules! ops_impl {
    ($($base:ident { $l:ident, $r:ident } => $ty:ty)*) => {$(
        impl std::ops::Add for $base<$ty> {
            type Output = Self;
            #[track_caller]
            fn add(self, rhs: Self) -> Self::Output {
                Self::new(self.$l + rhs.$l, self.$r + rhs.$r)
            }
        }
        impl std::ops::Sub for $base<$ty> {
            type Output = Self;
            #[track_caller]
            fn sub(self, rhs: Self) -> Self::Output {
                Self::new(self.$l - rhs.$l, self.$r - rhs.$r)
            }
        }
        impl std::ops::Mul for $base<$ty> {
            type Output = Self;
            #[track_caller]
            fn mul(self, rhs: Self) -> Self::Output {
                Self::new(self.$l * rhs.$l, self.$r * rhs.$r)
            }
        }
        impl std::ops::Div for $base<$ty> {
            type Output = Self;
            #[track_caller]
            fn div(self, rhs: Self) -> Self::Output {
                Self::new(self.$l / rhs.$l, self.$r / rhs.$r)
            }
        }

        impl std::ops::Add<$ty> for $base<$ty> {
            type Output = Self;
            #[track_caller]
            fn add(self, rhs: $ty) -> Self::Output {
                Self::new(self.$l + rhs, self.$r + rhs)
            }
        }
        impl std::ops::Sub<$ty> for $base<$ty> {
            type Output = Self;
            #[track_caller]
            fn sub(self, rhs: $ty) -> Self::Output {
                Self::new(self.$l - rhs, self.$r - rhs)
            }
        }
        impl std::ops::Mul<$ty> for $base<$ty> {
            type Output = Self;
            #[track_caller]
            fn mul(self, rhs: $ty) -> Self::Output {
                Self::new(self.$l * rhs, self.$r * rhs)
            }
        }
        impl std::ops::Div<$ty> for $base<$ty> {
            type Output = Self;
            #[track_caller]
            fn div(self, rhs: $ty) -> Self::Output {
                Self::new(self.$l / rhs, self.$r / rhs)
            }
        }

        impl std::ops::AddAssign for $base<$ty> {
            #[track_caller]
            fn add_assign(&mut self, rhs: Self) {
                *self = *self + rhs
            }
        }
        impl std::ops::SubAssign for $base<$ty> {
            #[track_caller]
            fn sub_assign(&mut self, rhs: Self) {
                *self = *self - rhs
            }
        }
        impl std::ops::MulAssign for $base<$ty> {
            #[track_caller]
            fn mul_assign(&mut self, rhs: Self) {
                *self = *self * rhs
            }
        }
        impl std::ops::DivAssign for $base<$ty> {
            #[track_caller]
            fn div_assign(&mut self, rhs: Self) {
                *self = *self / rhs
            }
        }
        impl std::ops::AddAssign<$ty> for $base<$ty> {
            #[track_caller]
            fn add_assign(&mut self, rhs: $ty) {
                *self = *self + rhs
            }
        }
        impl std::ops::SubAssign<$ty> for $base<$ty> {
            #[track_caller]
            fn sub_assign(&mut self, rhs: $ty) {
                *self = *self - rhs
            }
        }
        impl std::ops::MulAssign<$ty> for $base<$ty> {
            #[track_caller]
            fn mul_assign(&mut self, rhs: $ty) {
                *self = *self * rhs
            }
        }
        impl std::ops::DivAssign<$ty> for $base<$ty> {
            #[track_caller]
            fn div_assign(&mut self, rhs: $ty) {
                *self = *self / rhs
            }
        }

        impl From<($ty, $ty)> for $base<$ty> {
            fn from((l, r): ($ty, $ty)) -> Self {
                Self { $l: l, $r: r }
            }
        }
        impl From<[$ty; 2]> for $base<$ty> {
            fn from([l, r]: [$ty; 2]) -> Self {
                Self { $l: l, $r: r }
            }
        }
        impl From<$ty> for $base<$ty> {
            fn from(val: $ty) -> Self {
                Self { $l: val, $r: val }
            }
        }

        impl From<$base<$ty>> for ($ty, $ty) {
            fn from(this: $base<$ty>) -> Self {
                (this.$l, this.$r)
            }
        }
        impl From<$base<$ty>> for [$ty; 2] {
            fn from(this: $base<$ty>) -> Self {
                [this.$l, this.$r]
            }
        }
    )*};
}

pub(crate) use ops_impl;
