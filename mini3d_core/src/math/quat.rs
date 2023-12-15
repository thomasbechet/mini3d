use core::{
    fmt::Display,
    ops::{Mul, MulAssign},
};

use super::{
    fixed::{FixedPoint, RealFixedPoint, SignedFixedPoint, TrigFixedPoint, I32F16},
    mat::M4,
    vec::{V3, V4},
};

#[derive(Default, Debug, Copy, Clone)]
pub struct Q<T: FixedPoint> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T: FixedPoint + RealFixedPoint + SignedFixedPoint + TrigFixedPoint> Q<T> {
    pub const fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }

    pub fn from_axis_angle(axis: V3<T>, angle: T) -> Self {
        let half_angle = angle * T::HALF;
        let (sin, cos) = half_angle.sin_cos();
        Self {
            x: axis.x * sin,
            y: axis.y * sin,
            z: axis.z * sin,
            w: cos,
        }
    }

    pub fn mul_quat(self, q: Self) -> Self {
        Self::new(
            self.w * q.w - self.x * q.x - self.y * q.y - self.z * q.z,
            self.w * q.x + self.x * q.w + self.y * q.z - self.z * q.y,
            self.w * q.y + self.y * q.w + self.z * q.x - self.x * q.z,
            self.w * q.z + self.z * q.w + self.x * q.y - self.y * q.x,
        )
    }

    pub fn mul_vec3(self, v: V3<T>) -> V3<T> {
        let u = V3::new(self.x, self.y, self.z);
        u * u.dot(v) * T::TWO + v * (self.w * self.w - u.dot(u)) + u.cross(v) * self.w * T::TWO
    }

    pub fn to_mat4(mut self) -> M4<T> {
        let n = (self.x * self.x + self.y * self.y + self.z * self.z + self.w * self.w)
            .sqrt()
            .recip();

        self.x *= n;
        self.y *= n;
        self.z *= n;
        self.w *= n;

        M4::from_rows_element(
            (
                T::ONE - self.y * self.y * T::TWO - self.z * self.z * T::TWO,
                self.x * self.y * T::TWO - self.z * self.w * T::TWO,
                self.x * self.z * T::TWO + self.y * self.w * T::TWO,
                T::ZERO,
            ),
            (
                self.x * self.y * T::TWO + self.z * self.w * T::TWO,
                T::ONE - self.x * self.x * T::TWO - self.z * self.z * T::TWO,
                self.y * self.z * T::TWO - self.x * self.w * T::TWO,
                T::ZERO,
            ),
            (
                self.x * self.z * T::TWO - self.y * self.w * T::TWO,
                self.y * self.z * T::TWO + self.x * self.w * T::TWO,
                T::ONE - self.x * self.x * T::TWO - self.y * self.y * T::TWO,
                T::ZERO,
            ),
            (T::ZERO, T::ZERO, T::ZERO, T::ONE),
        )
    }

    pub fn to_axes(self) -> (V4<T>, V4<T>, V4<T>) {
        // Ensure normalize

        let x2 = self.x + self.x;
        let y2 = self.y + self.y;
        let z2 = self.z + self.z;
        let xx = self.x * x2;
        let xy = self.x * y2;
        let xz = self.x * z2;
        let yy = self.y * y2;
        let yz = self.y * z2;
        let zz = self.z * z2;
        let wx = self.w * x2;
        let wy = self.w * y2;
        let wz = self.w * z2;

        let x_axis = V4::new(T::ONE - (yy + zz), xy + wz, xz - wy, T::ZERO);
        let y_axis = V4::new(xy - wz, T::ONE - (xx + zz), yz + wx, T::ZERO);
        let z_axis = V4::new(xz + wy, yz - wx, T::ONE - (xx + yy), T::ZERO);
        (x_axis, y_axis, z_axis)
    }
}

impl<T: FixedPoint + RealFixedPoint + SignedFixedPoint + TrigFixedPoint> Mul for Q<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_quat(rhs)
    }
}

impl<T: FixedPoint + RealFixedPoint + SignedFixedPoint + TrigFixedPoint> MulAssign for Q<T> {
    fn mul_assign(&mut self, rhs: Self) {
        *self = self.mul_quat(rhs);
    }
}

impl<T: FixedPoint + RealFixedPoint + SignedFixedPoint + TrigFixedPoint> Mul<V3<T>> for Q<T> {
    type Output = V3<T>;

    fn mul(self, rhs: V3<T>) -> V3<T> {
        self.mul_vec3(rhs)
    }
}

impl<T: FixedPoint + Display> Display for Q<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[{}, {}, {}, {}]", self.x, self.y, self.z, self.w)
    }
}

pub type QI32F16 = Q<I32F16>;

#[cfg(test)]
mod test {
    use std::println;

    use crate::math::fixed::I32F16;

    use super::*;

    #[test]
    fn test_mat4() {
        let q = Q::<I32F16>::from_axis_angle(V3::Z, I32F16::PI / 2);
        println!("{}", q);
        let v = V3::X;
        println!("{}", q.mul_vec3(v));
    }
}
