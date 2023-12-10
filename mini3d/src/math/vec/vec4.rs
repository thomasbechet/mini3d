use core::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use crate::math::fixed::{FixedPoint, I32, I32F16, I32F24, U32, U32F16, U32F24};

use super::{V2, V3};

#[derive(Copy, Clone)]
pub struct V4<T: FixedPoint> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T: FixedPoint> V4<T> {
    pub const fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }

    pub const fn from_vec2(v: V2<T>, z: T, w: T) -> Self {
        Self::new(v.x, v.y, z, w)
    }

    pub const fn from_vec3(v: V3<T>, w: T) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }

    pub fn dot(self, rhs: Self) -> T {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z) + (self.w * rhs.w)
    }

    pub fn length_squared(self) -> T {
        self.dot(self)
    }

    pub fn length(self) -> T {
        self.length_squared().sqrt()
    }

    pub fn normalize(&mut self) {
        *self = self.normalized();
    }

    pub fn normalized(self) -> Self {
        let length = self.length();
        self / length
    }
}

impl<T: FixedPoint> From<T> for V4<T> {
    fn from(t: T) -> Self {
        Self::new(t, t, t, t)
    }
}

impl<T: FixedPoint> From<(T, T, T, T)> for V4<T> {
    fn from((x, y, z, w): (T, T, T, T)) -> Self {
        Self::new(x, y, z, w)
    }
}

impl<T: FixedPoint> From<(V2<T>, T, T)> for V4<T> {
    fn from((v, z, w): (V2<T>, T, T)) -> Self {
        Self::from_vec2(v, z, w)
    }
}

impl<T: FixedPoint> From<(V3<T>, T)> for V4<T> {
    fn from((v, w): (V3<T>, T)) -> Self {
        Self::from_vec3(v, w)
    }
}

impl<T: FixedPoint + Add<Output = T>> Add for V4<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x + rhs.x,
            self.y + rhs.y,
            self.z + rhs.z,
            self.w + rhs.w,
        )
    }
}

impl<T: FixedPoint + Add<Output = T>> Add<T> for V4<T> {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs, self.z + rhs, self.w + rhs)
    }
}

impl<T: FixedPoint + Sub<Output = T>> Sub for V4<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x - rhs.x,
            self.y - rhs.y,
            self.z - rhs.z,
            self.w - rhs.w,
        )
    }
}

impl<T: FixedPoint + Sub<Output = T>> Sub<T> for V4<T> {
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self::new(self.x - rhs, self.y - rhs, self.z - rhs, self.w - rhs)
    }
}

impl<T: FixedPoint + Mul<Output = T>> Mul for V4<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x * rhs.x,
            self.y * rhs.y,
            self.z * rhs.z,
            self.w * rhs.w,
        )
    }
}

impl<T: FixedPoint + Mul<Output = T>> Mul<T> for V4<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl<T: FixedPoint + Div<Output = T>> Div for V4<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(
            self.x / rhs.x,
            self.y / rhs.y,
            self.z / rhs.z,
            self.w / rhs.w,
        )
    }
}

impl<T: FixedPoint + Div<Output = T>> Div<T> for V4<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs, self.w / rhs)
    }
}

impl<T: FixedPoint + Display> Display for V4<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{}, {}, {}, {}]", self.x, self.y, self.z, self.w)
    }
}

pub type V4I32 = V4<I32>;
pub type V4U32 = V4<U32>;
pub type V4I32F16 = V4<I32F16>;
pub type V4U32F16 = V4<U32F16>;
pub type V4I32F24 = V4<I32F24>;
pub type V4U32F24 = V4<U32F24>;

#[cfg(test)]
mod test {
    use std::println;

    use mini3d_derive::fixed;

    use super::*;

    #[test]
    fn test_vec4() {
        println!("{}", I32F24::EPSILON);
        let x = V4I32F24::from(fixed!(1i32f24));
        println!("{}", x + x / fixed!(0.2123i32f24));
    }
}
