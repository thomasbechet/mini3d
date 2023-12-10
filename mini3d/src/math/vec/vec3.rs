use core::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use crate::math::fixed::{FixedPoint, I32, I32F16, I32F24, U32, U32F16, U32F24};

use super::{V2, V4};

#[derive(Copy, Clone)]
pub struct V3<T: FixedPoint> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: FixedPoint> V3<T> {
    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub const fn from_vec2(v: V2<T>, z: T) -> Self {
        Self::new(v.x, v.y, z)
    }

    pub const fn from_vec4(v: super::V4<T>) -> Self {
        Self::new(v.x, v.y, v.z)
    }

    pub fn dot(self, rhs: Self) -> T {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
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

    pub fn cross(self, rhs: Self) -> Self {
        Self::new(
            self.x * rhs.y - self.y * rhs.x,
            self.y * rhs.z - self.z * rhs.y,
            self.z * rhs.x - self.x * rhs.z,
        )
    }
}

impl<T: FixedPoint> From<T> for V3<T> {
    fn from(t: T) -> Self {
        Self::new(t, t, t)
    }
}

impl<T: FixedPoint> From<(T, T, T)> for V3<T> {
    fn from((x, y, z): (T, T, T)) -> Self {
        Self::new(x, y, z)
    }
}

impl<T: FixedPoint> From<(V2<T>, T)> for V3<T> {
    fn from((v, z): (V2<T>, T)) -> Self {
        Self::from_vec2(v, z)
    }
}

impl<T: FixedPoint> From<V4<T>> for V3<T> {
    fn from(v: V4<T>) -> Self {
        Self::from_vec4(v)
    }
}

impl<T: FixedPoint + Add<Output = T>> Add for V3<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T: FixedPoint + Add<Output = T>> Add<T> for V3<T> {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl<T: FixedPoint + Sub<Output = T>> Sub for V3<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T: FixedPoint + Sub<Output = T>> Sub<T> for V3<T> {
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}

impl<T: FixedPoint + Mul<Output = T>> Mul for V3<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl<T: FixedPoint + Mul<Output = T>> Mul<T> for V3<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<T: FixedPoint + Div<Output = T>> Div for V3<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl<T: FixedPoint + Div<Output = T>> Div<T> for V3<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl<T: FixedPoint + Display> Display for V3<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{}, {}, {}]", self.x, self.y, self.z)
    }
}

pub type V3I32 = V3<I32>;
pub type V3U32 = V3<U32>;
pub type V3I32F16 = V3<I32F16>;
pub type V3U32F16 = V3<U32F16>;
pub type V3I32F24 = V3<I32F24>;
pub type V3U32F24 = V3<U32F24>;

#[cfg(test)]
mod test {
    use std::println;

    use mini3d_derive::fixed;

    use super::*;

    #[test]
    fn test_vec3() {
        println!("{}", I32F24::EPSILON);
        let x = V3I32F24::from(fixed!(1i32f24));
        println!("{}", x + x / fixed!(0.2123i32f24));
    }
}
