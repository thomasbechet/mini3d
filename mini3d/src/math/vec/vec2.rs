use core::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use crate::math::fixed::{FixedPoint, RealFixedPoint};

use super::{V3, V4};

#[derive(Default, Debug, Copy, Clone)]
pub struct V2<T: FixedPoint> {
    pub x: T,
    pub y: T,
}

impl<T: FixedPoint> V2<T> {
    pub const fn new(x: T, y: T) -> Self {
        Self { x, y }
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

    pub fn min(self, rhs: Self) -> Self {
        Self::new(self.x.min(rhs.x), self.y.min(rhs.y))
    }

    pub fn max(self, rhs: Self) -> Self {
        Self::new(self.x.max(rhs.x), self.y.max(rhs.y))
    }
}

impl<T: FixedPoint + RealFixedPoint> V2<T> {
    pub fn length(self) -> T {
        self.length_squared().sqrt()
    }

    pub fn normalize(self) -> Self {
        self / self.length()
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

#[cfg(test)]
mod test {
    use std::println;

    use mini3d_derive::fixed;

    use crate::math::{fixed::I32F24, vec::V2I32F24};

    #[test]
    fn test_vec2() {
        println!("{}", I32F24::EPSILON);
        let x = V2I32F24::from(fixed!(1i32f24));
        println!("{}", x.normalize().length());
    }
}
