use core::{fmt::Display, ops::Mul};

use crate::math::{
    fixed::{FixedPoint, SignedFixedPoint, TrigFixedPoint},
    quat::Q,
    vec::{V3, V4},
};

// Column-major matrix (follow math convention)
pub struct M4<T: FixedPoint> {
    pub xaxis: V4<T>,
    pub yaxis: V4<T>,
    pub zaxis: V4<T>,
    pub waxis: V4<T>,
}

impl<T: FixedPoint> M4<T> {
    pub const IDENTITY: Self = Self::from_rows_element(
        (T::ONE, T::ZERO, T::ZERO, T::ZERO),
        (T::ZERO, T::ONE, T::ZERO, T::ZERO),
        (T::ZERO, T::ZERO, T::ONE, T::ZERO),
        (T::ZERO, T::ZERO, T::ZERO, T::ONE),
    );

    pub const fn from_cols(colx: V4<T>, coly: V4<T>, colz: V4<T>, colw: V4<T>) -> Self {
        Self {
            xaxis: colx,
            yaxis: coly,
            zaxis: colz,
            waxis: colw,
        }
    }

    pub const fn from_rows(rowx: V4<T>, rowy: V4<T>, rowz: V4<T>, roww: V4<T>) -> Self {
        Self {
            xaxis: V4::new(rowx.x, rowy.x, rowz.x, roww.x),
            yaxis: V4::new(rowx.y, rowy.y, rowz.y, roww.y),
            zaxis: V4::new(rowx.z, rowy.z, rowz.z, roww.z),
            waxis: V4::new(rowx.w, rowy.w, rowz.w, roww.w),
        }
    }

    pub const fn from_rows_element(
        rowx: (T, T, T, T),
        rowy: (T, T, T, T),
        rowz: (T, T, T, T),
        roww: (T, T, T, T),
    ) -> Self {
        Self {
            xaxis: V4::new(rowx.0, rowy.0, rowz.0, roww.0),
            yaxis: V4::new(rowx.1, rowy.1, rowz.1, roww.1),
            zaxis: V4::new(rowx.2, rowy.2, rowz.2, roww.2),
            waxis: V4::new(rowx.3, rowy.3, rowz.3, roww.3),
        }
    }

    pub const fn translate(x: T, y: T, z: T) -> Self {
        Self::from_rows_element(
            (T::ONE, T::ZERO, T::ZERO, x),
            (T::ZERO, T::ONE, T::ZERO, y),
            (T::ZERO, T::ZERO, T::ONE, z),
            (T::ZERO, T::ZERO, T::ZERO, T::ONE),
        )
    }

    pub const fn scale(x: T, y: T, z: T) -> Self {
        Self::from_rows_element(
            (x, T::ZERO, T::ZERO, T::ZERO),
            (T::ZERO, y, T::ZERO, T::ZERO),
            (T::ZERO, T::ZERO, z, T::ZERO),
            (T::ZERO, T::ZERO, T::ZERO, T::ONE),
        )
    }

    pub fn mul_mat4(&self, rhs: &Self) -> Self {
        let colx = V4::new(
            self.xaxis.x * rhs.xaxis.x
                + self.yaxis.x * rhs.xaxis.y
                + self.zaxis.x * rhs.xaxis.z
                + self.waxis.x * rhs.xaxis.w,
            self.xaxis.x * rhs.yaxis.x
                + self.yaxis.x * rhs.yaxis.y
                + self.zaxis.x * rhs.yaxis.z
                + self.waxis.x * rhs.yaxis.w,
            self.xaxis.x * rhs.zaxis.x
                + self.yaxis.x * rhs.zaxis.y
                + self.zaxis.x * rhs.zaxis.z
                + self.waxis.x * rhs.zaxis.w,
            self.xaxis.x * rhs.waxis.x
                + self.yaxis.x * rhs.waxis.y
                + self.zaxis.x * rhs.waxis.z
                + self.waxis.x * rhs.waxis.w,
        );

        let coly = V4::new(
            self.xaxis.y * rhs.xaxis.x
                + self.yaxis.y * rhs.xaxis.y
                + self.zaxis.y * rhs.xaxis.z
                + self.waxis.y * rhs.xaxis.w,
            self.xaxis.y * rhs.yaxis.x
                + self.yaxis.y * rhs.yaxis.y
                + self.zaxis.y * rhs.yaxis.z
                + self.waxis.y * rhs.yaxis.w,
            self.xaxis.y * rhs.zaxis.x
                + self.yaxis.y * rhs.zaxis.y
                + self.zaxis.y * rhs.zaxis.z
                + self.waxis.y * rhs.zaxis.w,
            self.xaxis.y * rhs.waxis.x
                + self.yaxis.y * rhs.waxis.y
                + self.zaxis.y * rhs.waxis.z
                + self.waxis.y * rhs.waxis.w,
        );

        let colz = V4::new(
            self.xaxis.z * rhs.xaxis.x
                + self.yaxis.z * rhs.xaxis.y
                + self.zaxis.z * rhs.xaxis.z
                + self.waxis.z * rhs.xaxis.w,
            self.xaxis.z * rhs.yaxis.x
                + self.yaxis.z * rhs.yaxis.y
                + self.zaxis.z * rhs.yaxis.z
                + self.waxis.z * rhs.yaxis.w,
            self.xaxis.z * rhs.zaxis.x
                + self.yaxis.z * rhs.zaxis.y
                + self.zaxis.z * rhs.zaxis.z
                + self.waxis.z * rhs.zaxis.w,
            self.xaxis.z * rhs.waxis.x
                + self.yaxis.z * rhs.waxis.y
                + self.zaxis.z * rhs.waxis.z
                + self.waxis.z * rhs.waxis.w,
        );

        let colw = V4::new(
            self.xaxis.w * rhs.xaxis.x
                + self.yaxis.w * rhs.xaxis.y
                + self.zaxis.w * rhs.xaxis.z
                + self.waxis.w * rhs.xaxis.w,
            self.xaxis.w * rhs.yaxis.x
                + self.yaxis.w * rhs.yaxis.y
                + self.zaxis.w * rhs.yaxis.z
                + self.waxis.w * rhs.yaxis.w,
            self.xaxis.w * rhs.zaxis.x
                + self.yaxis.w * rhs.zaxis.y
                + self.zaxis.w * rhs.zaxis.z
                + self.waxis.w * rhs.zaxis.w,
            self.xaxis.w * rhs.waxis.x
                + self.yaxis.w * rhs.waxis.y
                + self.zaxis.w * rhs.waxis.z
                + self.waxis.w * rhs.waxis.w,
        );
        Self::from_cols(colx, coly, colz, colw)
    }

    pub fn mul_vec4(&self, rhs: &V4<T>) -> V4<T> {
        V4::new(
            self.xaxis.x * rhs.x
                + self.yaxis.x * rhs.y
                + self.zaxis.x * rhs.z
                + self.waxis.x * rhs.w,
            self.xaxis.y * rhs.x
                + self.yaxis.y * rhs.y
                + self.zaxis.y * rhs.z
                + self.waxis.y * rhs.w,
            self.xaxis.z * rhs.x
                + self.yaxis.z * rhs.y
                + self.zaxis.z * rhs.z
                + self.waxis.z * rhs.w,
            self.xaxis.w * rhs.x
                + self.yaxis.w * rhs.y
                + self.zaxis.w * rhs.z
                + self.waxis.w * rhs.w,
        )
    }

    pub fn div(&self, rhs: T) -> Self {
        Self::from_cols(
            self.xaxis / rhs,
            self.yaxis / rhs,
            self.zaxis / rhs,
            self.waxis / rhs,
        )
    }
}

impl<T: FixedPoint + TrigFixedPoint + SignedFixedPoint> M4<T> {
    pub fn from_scale_rotation_translation(
        scale: V3<T>,
        rotation: Q<T>,
        translation: V3<T>,
    ) -> Self {
        let (x_axis, y_axis, z_axis) = rotation.to_axes();
        Self::from_cols(
            x_axis.mul(scale.x),
            y_axis.mul(scale.y),
            z_axis.mul(scale.z),
            V4::from_vec3(translation, T::ONE),
        )
    }

    pub fn rotate_x(angle: T) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::from_rows_element(
            (T::ONE, T::ZERO, T::ZERO, T::ZERO),
            (T::ZERO, cos, -sin, T::ZERO),
            (T::ZERO, sin, cos, T::ZERO),
            (T::ZERO, T::ZERO, T::ZERO, T::ONE),
        )
    }

    pub fn rotate_y(angle: T) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::from_rows_element(
            (cos, T::ZERO, sin, T::ZERO),
            (T::ZERO, T::ONE, T::ZERO, T::ZERO),
            (-sin, T::ZERO, cos, T::ZERO),
            (T::ZERO, T::ZERO, T::ZERO, T::ONE),
        )
    }

    pub fn rotate_z(angle: T) -> Self {
        let (sin, cos) = angle.sin_cos();
        Self::from_rows_element(
            (cos, -sin, T::ZERO, T::ZERO),
            (sin, cos, T::ZERO, T::ZERO),
            (T::ZERO, T::ZERO, T::ONE, T::ZERO),
            (T::ZERO, T::ZERO, T::ZERO, T::ONE),
        )
    }

    pub fn rotate(x: T, y: T, z: T) -> Self {
        let mut mat = Self::IDENTITY;
        if x != T::ZERO {
            mat = mat * Self::rotate_x(x);
        }
        if y != T::ZERO {
            mat = mat * Self::rotate_y(y);
        }
        if z != T::ZERO {
            mat = mat * Self::rotate_z(z);
        }
        mat
    }

    pub fn view(eye: V3<T>, forward: V3<T>, up: V3<T>) -> Self {
        let zaxis = forward.normalize();
        let xaxis = up.cross(zaxis).normalize();
        let yaxis = zaxis.cross(xaxis);

        Self::from_rows_element(
            (xaxis.x, yaxis.x, zaxis.x, T::ZERO),
            (xaxis.y, yaxis.y, zaxis.y, T::ZERO),
            (xaxis.z, yaxis.z, zaxis.z, T::ZERO),
            (-eye.dot(xaxis), -eye.dot(yaxis), -eye.dot(zaxis), T::ONE),
        )
    }

    pub fn inverse(&self) -> Self {
        let a2323 = self.zaxis.z * self.waxis.w - self.zaxis.w * self.waxis.z;
        let a1323 = self.zaxis.y * self.waxis.w - self.zaxis.w * self.waxis.y;
        let a1223 = self.zaxis.y * self.waxis.z - self.zaxis.z * self.waxis.y;
        let a0323 = self.zaxis.x * self.waxis.w - self.zaxis.w * self.waxis.x;
        let a0223 = self.zaxis.x * self.waxis.z - self.zaxis.z * self.waxis.x;
        let a0123 = self.zaxis.x * self.waxis.y - self.zaxis.y * self.waxis.x;
        let a2313 = self.yaxis.z * self.waxis.w - self.yaxis.w * self.waxis.z;
        let a1313 = self.yaxis.y * self.waxis.w - self.yaxis.w * self.waxis.y;
        let a1213 = self.yaxis.y * self.waxis.z - self.yaxis.z * self.waxis.y;
        let a2312 = self.yaxis.z * self.zaxis.w - self.yaxis.w * self.zaxis.z;
        let a1312 = self.yaxis.y * self.zaxis.w - self.yaxis.w * self.zaxis.y;
        let a1212 = self.yaxis.y * self.zaxis.z - self.yaxis.z * self.zaxis.y;
        let a0313 = self.yaxis.x * self.waxis.w - self.yaxis.w * self.waxis.x;
        let a0213 = self.yaxis.x * self.waxis.z - self.yaxis.z * self.waxis.x;
        let a0312 = self.yaxis.x * self.zaxis.w - self.yaxis.w * self.zaxis.x;
        let a0212 = self.yaxis.x * self.zaxis.z - self.yaxis.z * self.zaxis.x;
        let a0113 = self.yaxis.x * self.waxis.y - self.yaxis.y * self.waxis.x;
        let a0112 = self.yaxis.x * self.zaxis.y - self.yaxis.y * self.zaxis.x;

        let det = self.xaxis.x
            * (self.yaxis.y * a2323 - self.yaxis.z * a1323 + self.yaxis.w * a1223)
            - self.xaxis.y * (self.yaxis.x * a2323 - self.yaxis.z * a0323 + self.yaxis.w * a0223)
            + self.xaxis.z * (self.yaxis.x * a1323 - self.yaxis.y * a0323 + self.yaxis.w * a0123)
            - self.xaxis.w * (self.yaxis.x * a1223 - self.yaxis.y * a0223 + self.yaxis.z * a0123);

        if det == T::ZERO {
            panic!("try to invert non-invertible matrix");
        }

        let inv_det = T::ONE / det;

        let colx = V4::new(
            (self.yaxis.y * a2323 - self.yaxis.z * a1323 + self.yaxis.w * a1223) * inv_det,
            -(self.xaxis.y * a2323 - self.xaxis.z * a1323 + self.xaxis.w * a1223) * inv_det,
            (self.xaxis.y * a2313 - self.xaxis.z * a1313 + self.xaxis.w * a1213) * inv_det,
            -(self.xaxis.y * a2312 - self.xaxis.z * a1312 + self.xaxis.w * a1212) * inv_det,
        );

        let coly = V4::new(
            -(self.yaxis.x * a2323 - self.yaxis.z * a0323 + self.yaxis.w * a0223) * inv_det,
            (self.xaxis.x * a2323 - self.xaxis.z * a0323 + self.xaxis.w * a0223) * inv_det,
            -(self.xaxis.x * a2313 - self.xaxis.z * a0313 + self.xaxis.w * a0213) * inv_det,
            (self.xaxis.x * a2312 - self.xaxis.z * a0312 + self.xaxis.w * a0212) * inv_det,
        );

        let colz = V4::new(
            (self.yaxis.x * a1323 - self.yaxis.y * a0323 + self.yaxis.w * a0123) * inv_det,
            -(self.xaxis.x * a1323 - self.xaxis.y * a0323 + self.xaxis.w * a0123) * inv_det,
            (self.xaxis.x * a1313 - self.xaxis.y * a0313 + self.xaxis.w * a0113) * inv_det,
            -(self.xaxis.x * a1312 - self.xaxis.y * a0312 + self.xaxis.w * a0112) * inv_det,
        );

        let colw = V4::new(
            -(self.yaxis.x * a1223 - self.yaxis.y * a0223 + self.yaxis.z * a0123) * inv_det,
            (self.xaxis.x * a1223 - self.xaxis.y * a0223 + self.xaxis.z * a0123) * inv_det,
            -(self.xaxis.x * a1213 - self.xaxis.y * a0213 + self.xaxis.z * a0113) * inv_det,
            (self.xaxis.x * a1212 - self.xaxis.y * a0212 + self.xaxis.z * a0112) * inv_det,
        );

        Self::from_cols(colx, coly, colz, colw)
    }

    pub const fn transposed(&self) -> Self {
        Self::from_rows_element(
            (self.xaxis.x, self.xaxis.y, self.xaxis.z, self.xaxis.w),
            (self.yaxis.x, self.yaxis.y, self.yaxis.z, self.yaxis.w),
            (self.zaxis.x, self.zaxis.y, self.zaxis.z, self.zaxis.w),
            (self.waxis.x, self.waxis.y, self.waxis.z, self.waxis.w),
        )
    }

    pub fn transpose(&mut self) {
        *self = self.transposed();
    }
}

impl<T: FixedPoint> Mul<Self> for M4<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.mul_mat4(&rhs)
    }
}

impl<T: FixedPoint> Mul<V4<T>> for M4<T> {
    type Output = V4<T>;

    fn mul(self, rhs: V4<T>) -> Self::Output {
        self.mul_vec4(&rhs)
    }
}

impl<T: FixedPoint + Display> Display for M4<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(
            f,
            "[{}, {}, {}, {}]",
            self.xaxis.x, self.yaxis.x, self.zaxis.x, self.waxis.x
        )?;
        writeln!(
            f,
            "[{}, {}, {}, {}]",
            self.xaxis.y, self.yaxis.y, self.zaxis.y, self.waxis.y
        )?;
        writeln!(
            f,
            "[{}, {}, {}, {}]",
            self.xaxis.z, self.yaxis.z, self.zaxis.z, self.waxis.z
        )?;
        writeln!(
            f,
            "[{}, {}, {}, {}]",
            self.xaxis.w, self.yaxis.w, self.zaxis.w, self.waxis.w
        )
    }
}

#[cfg(test)]
mod test {
    use std::println;

    use mini3d_derive::fixed;

    use crate::math::fixed::I32F16;

    use super::*;

    #[test]
    fn test_mat4() {
        let t = M4::<I32F16>::translate(fixed!(1), fixed!(2), fixed!(3));
        println!("{}", t);
        println!("{}", t.transposed());
    }
}
