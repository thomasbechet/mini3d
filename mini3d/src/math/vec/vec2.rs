use core::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use crate::math::fixed::{FixedPoint, I32, I32F16, I32F24, U32, U32F16, U32F24};

use super::{V3, V4};

#[derive(Copy, Clone)]
pub struct V2<T: FixedPoint> {
    pub x: T,
    pub y: T,
}

impl<T: FixedPoint> V2<T> {
    pub const fn new(x: T, y: T) -> Self {
        V2 { x, y }
    }

    pub const fn from_vec3(v: V3<T>) -> Self {
        Self::new(v.x, v.y)
    }

    pub const fn from_vec4(v: V4<T>) -> Self {
        Self::new(v.x, v.y)
    }

    pub fn dot(self, rhs: Self) -> T {
        (self.x * rhs.x) + (self.y * rhs.y)
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

impl<T: FixedPoint> From<T> for V2<T> {
    fn from(t: T) -> Self {
        V2::new(t, t)
    }
}

impl<T: FixedPoint> From<(T, T)> for V2<T> {
    fn from((x, y): (T, T)) -> Self {
        V2::new(x, y)
    }
}

impl<T: FixedPoint> From<V3<T>> for V2<T> {
    fn from(v: V3<T>) -> Self {
        Self::from_vec3(v)
    }
}

impl<T: FixedPoint> From<V4<T>> for V2<T> {
    fn from(v: V4<T>) -> Self {
        Self::from_vec4(v)
    }
}

impl<T: FixedPoint + Add<Output = T>> Add for V2<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        V2::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl<T: FixedPoint + Add<Output = T>> Add<T> for V2<T> {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        V2::new(self.x + rhs, self.y + rhs)
    }
}

impl<T: FixedPoint + Sub<Output = T>> Sub for V2<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        V2::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: FixedPoint + Sub<Output = T>> Sub<T> for V2<T> {
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        V2::new(self.x - rhs, self.y - rhs)
    }
}

impl<T: FixedPoint + Mul<Output = T>> Mul for V2<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        V2::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl<T: FixedPoint + Mul<Output = T>> Mul<T> for V2<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        V2::new(self.x * rhs, self.y * rhs)
    }
}

impl<T: FixedPoint + Div<Output = T>> Div for V2<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        V2::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl<T: FixedPoint + Div<Output = T>> Div<T> for V2<T> {
    type Output = Self;

    fn div(self, rhs: T) -> Self::Output {
        V2::new(self.x / rhs, self.y / rhs)
    }
}

impl<T: FixedPoint + Display> Display for V2<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{}, {}]", self.x, self.y)
    }
}

pub type V2I32 = V2<I32>;
pub type V2U32 = V2<U32>;
pub type V2I32F16 = V2<I32F16>;
pub type V2U32F16 = V2<U32F16>;
pub type V2I32F24 = V2<I32F24>;
pub type V2U32F24 = V2<U32F24>;

#[cfg(test)]
mod test {
    use std::println;

    use mini3d_derive::fixed;

    use super::*;

    #[test]
    fn test_vec2() {
        println!("{}", I32F24::EPSILON);
        let x = V2I32F24::from(fixed!(1i32f24));
        println!("{}", x.normalized().length());
    }
}
