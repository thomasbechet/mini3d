use core::{fmt::Display, ops::Mul};

use crate::math::{
    fixed::{FixedPoint, SignedFixedPoint, TrigFixedPoint},
    vec::{V3, V4},
};

// Column-major matrix (follow math convention)
pub struct M4<T: FixedPoint> {
    pub colx: V4<T>,
    pub coly: V4<T>,
    pub colz: V4<T>,
    pub colw: V4<T>,
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
            colx,
            coly,
            colz,
            colw,
        }
    }

    pub const fn from_rows(rowx: V4<T>, rowy: V4<T>, rowz: V4<T>, roww: V4<T>) -> Self {
        Self {
            colx: V4::new(rowx.x, rowy.x, rowz.x, roww.x),
            coly: V4::new(rowx.y, rowy.y, rowz.y, roww.y),
            colz: V4::new(rowx.z, rowy.z, rowz.z, roww.z),
            colw: V4::new(rowx.w, rowy.w, rowz.w, roww.w),
        }
    }

    pub const fn from_rows_element(
        rowx: (T, T, T, T),
        rowy: (T, T, T, T),
        rowz: (T, T, T, T),
        roww: (T, T, T, T),
    ) -> Self {
        Self {
            colx: V4::new(rowx.0, rowy.0, rowz.0, roww.0),
            coly: V4::new(rowx.1, rowy.1, rowz.1, roww.1),
            colz: V4::new(rowx.2, rowy.2, rowz.2, roww.2),
            colw: V4::new(rowx.3, rowy.3, rowz.3, roww.3),
        }
    }

    pub const fn translate(x: T, y: T, z: T) -> Self {
        Self {
            colx: V4::new(T::ONE, T::ZERO, T::ZERO, T::ZERO),
            coly: V4::new(T::ZERO, T::ONE, T::ZERO, T::ZERO),
            colz: V4::new(T::ZERO, T::ZERO, T::ONE, T::ZERO),
            colw: V4::new(x, y, z, T::ONE),
        }
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
            self.colx.x * rhs.colx.x
                + self.coly.x * rhs.colx.y
                + self.colz.x * rhs.colx.z
                + self.colw.x * rhs.colx.w,
            self.colx.x * rhs.coly.x
                + self.coly.x * rhs.coly.y
                + self.colz.x * rhs.coly.z
                + self.colw.x * rhs.coly.w,
            self.colx.x * rhs.colz.x
                + self.coly.x * rhs.colz.y
                + self.colz.x * rhs.colz.z
                + self.colw.x * rhs.colz.w,
            self.colx.x * rhs.colw.x
                + self.coly.x * rhs.colw.y
                + self.colz.x * rhs.colw.z
                + self.colw.x * rhs.colw.w,
        );

        let coly = V4::new(
            self.colx.y * rhs.colx.x
                + self.coly.y * rhs.colx.y
                + self.colz.y * rhs.colx.z
                + self.colw.y * rhs.colx.w,
            self.colx.y * rhs.coly.x
                + self.coly.y * rhs.coly.y
                + self.colz.y * rhs.coly.z
                + self.colw.y * rhs.coly.w,
            self.colx.y * rhs.colz.x
                + self.coly.y * rhs.colz.y
                + self.colz.y * rhs.colz.z
                + self.colw.y * rhs.colz.w,
            self.colx.y * rhs.colw.x
                + self.coly.y * rhs.colw.y
                + self.colz.y * rhs.colw.z
                + self.colw.y * rhs.colw.w,
        );

        let colz = V4::new(
            self.colx.z * rhs.colx.x
                + self.coly.z * rhs.colx.y
                + self.colz.z * rhs.colx.z
                + self.colw.z * rhs.colx.w,
            self.colx.z * rhs.coly.x
                + self.coly.z * rhs.coly.y
                + self.colz.z * rhs.coly.z
                + self.colw.z * rhs.coly.w,
            self.colx.z * rhs.colz.x
                + self.coly.z * rhs.colz.y
                + self.colz.z * rhs.colz.z
                + self.colw.z * rhs.colz.w,
            self.colx.z * rhs.colw.x
                + self.coly.z * rhs.colw.y
                + self.colz.z * rhs.colw.z
                + self.colw.z * rhs.colw.w,
        );

        let colw = V4::new(
            self.colx.w * rhs.colx.x
                + self.coly.w * rhs.colx.y
                + self.colz.w * rhs.colx.z
                + self.colw.w * rhs.colx.w,
            self.colx.w * rhs.coly.x
                + self.coly.w * rhs.coly.y
                + self.colz.w * rhs.coly.z
                + self.colw.w * rhs.coly.w,
            self.colx.w * rhs.colz.x
                + self.coly.w * rhs.colz.y
                + self.colz.w * rhs.colz.z
                + self.colw.w * rhs.colz.w,
            self.colx.w * rhs.colw.x
                + self.coly.w * rhs.colw.y
                + self.colz.w * rhs.colw.z
                + self.colw.w * rhs.colw.w,
        );
        Self::from_cols(colx, coly, colz, colw)
    }

    pub fn mul_vec4(&self, rhs: &V4<T>) -> V4<T> {
        V4::new(
            self.colx.x * rhs.x + self.coly.x * rhs.y + self.colz.x * rhs.z + self.colw.x * rhs.w,
            self.colx.y * rhs.x + self.coly.y * rhs.y + self.colz.y * rhs.z + self.colw.y * rhs.w,
            self.colx.z * rhs.x + self.coly.z * rhs.y + self.colz.z * rhs.z + self.colw.z * rhs.w,
            self.colx.w * rhs.x + self.coly.w * rhs.y + self.colz.w * rhs.z + self.colw.w * rhs.w,
        )
    }

    pub fn div(&self, rhs: T) -> Self {
        Self::from_cols(
            self.colx / rhs,
            self.coly / rhs,
            self.colz / rhs,
            self.colw / rhs,
        )
    }
}

impl<T: FixedPoint + TrigFixedPoint + SignedFixedPoint> M4<T> {
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
        let zaxis = forward.normalized();
        let xaxis = up.cross(zaxis).normalized();
        let yaxis = zaxis.cross(xaxis);

        Self::from_rows_element(
            (xaxis.x, yaxis.x, zaxis.x, T::ZERO),
            (xaxis.y, yaxis.y, zaxis.y, T::ZERO),
            (xaxis.z, yaxis.z, zaxis.z, T::ZERO),
            (-eye.dot(xaxis), -eye.dot(yaxis), -eye.dot(zaxis), T::ONE),
        )
    }

    pub fn inverse(&self) -> Self {
        let inv_colx = V4::new(
            self.coly.y * self.colz.z * self.colw.w
                - self.coly.y * self.colz.w * self.colw.z
                - self.coly.z * self.colz.y * self.colw.w
                + self.coly.z * self.colz.w * self.colw.y
                + self.coly.w * self.colz.y * self.colw.z
                - self.coly.w * self.colz.z * self.colw.y,
            -self.colx.y * self.colz.z * self.colw.w
                + self.colx.y * self.colz.w * self.colw.z
                + self.colx.z * self.colz.y * self.colw.w
                - self.colx.z * self.colz.w * self.colw.y
                - self.colx.w * self.colz.y * self.colw.z
                + self.colx.w * self.colz.z * self.colw.y,
            self.colx.y * self.coly.z * self.colw.w
                - self.colx.y * self.coly.w * self.colw.z
                - self.colx.z * self.coly.y * self.colw.w
                + self.colx.z * self.coly.w * self.colw.y
                + self.colx.w * self.coly.y * self.colw.z
                - self.colx.w * self.coly.z * self.colw.y,
            -self.colx.y * self.coly.z * self.colz.w
                + self.colx.y * self.coly.w * self.colz.z
                + self.colx.z * self.coly.y * self.colz.w
                - self.colx.z * self.coly.w * self.colz.y
                - self.colx.w * self.coly.y * self.colz.z
                + self.colx.w * self.coly.z * self.colz.y,
        );

        let inv_coly = V4::new(
            -self.coly.x * self.colz.z * self.colw.w
                + self.coly.x * self.colz.w * self.colw.z
                + self.coly.z * self.colz.x * self.colw.w
                - self.coly.z * self.colz.w * self.colw.x
                - self.coly.w * self.colz.x * self.colw.z
                + self.coly.w * self.colz.z * self.colw.x,
            self.colx.x * self.colz.z * self.colw.w
                - self.colx.x * self.colz.w * self.colw.z
                - self.colx.z * self.colz.x * self.colw.w
                + self.colx.z * self.colz.w * self.colw.x
                + self.colx.w * self.colz.x * self.colw.z
                - self.colx.w * self.colz.z * self.colw.x,
            -self.colx.x * self.coly.z * self.colw.w
                + self.colx.x * self.coly.w * self.colw.z
                + self.colx.z * self.coly.x * self.colw.w
                - self.colx.z * self.coly.w * self.colw.x
                - self.colx.w * self.coly.x * self.colw.z
                + self.colx.w * self.coly.z * self.colw.x,
            self.colx.x * self.coly.z * self.colz.w
                - self.colx.x * self.coly.w * self.colz.z
                - self.colx.z * self.coly.x * self.colz.w
                + self.colx.z * self.coly.w * self.colz.x
                + self.colx.w * self.coly.x * self.colz.z
                - self.colx.w * self.coly.z * self.colz.x,
        );

        let inv_colz = V4::new(
            self.coly.x * self.colz.y * self.colw.w
                - self.coly.x * self.colz.w * self.colw.y
                - self.coly.y * self.colz.x * self.colw.w
                + self.coly.y * self.colz.w * self.colw.x
                + self.coly.w * self.colz.x * self.colw.y
                - self.coly.w * self.colz.y * self.colw.x,
            -self.colx.x * self.colz.y * self.colw.w
                + self.colx.x * self.colz.w * self.colw.y
                + self.colx.y * self.colz.x * self.colw.w
                - self.colx.y * self.colz.w * self.colw.x
                - self.colx.w * self.colz.x * self.colw.y
                + self.colx.w * self.colz.y * self.colw.x,
            self.colx.x * self.coly.y * self.colw.w
                - self.colx.x * self.coly.w * self.colw.y
                - self.colx.y * self.coly.x * self.colw.w
                + self.colx.y * self.coly.w * self.colw.x
                + self.colx.w * self.coly.x * self.colw.y
                - self.colx.w * self.coly.y * self.colw.x,
            -self.colx.x * self.coly.y * self.colz.w
                + self.colx.x * self.coly.w * self.colz.y
                + self.colx.y * self.coly.x * self.colz.w
                - self.colx.y * self.coly.w * self.colz.x
                - self.colx.w * self.coly.x * self.colz.y
                + self.colx.w * self.coly.y * self.colz.x,
        );

        let inv_colw = V4::new(
            -self.coly.x * self.colz.y * self.colw.z
                + self.coly.x * self.colz.z * self.colw.y
                + self.coly.y * self.colz.x * self.colw.z
                - self.coly.y * self.colz.z * self.colw.x
                - self.coly.z * self.colz.x * self.colw.y
                + self.coly.z * self.colz.y * self.colw.x,
            self.colx.x * self.colz.y * self.colw.z
                - self.colx.x * self.colz.z * self.colw.y
                - self.colx.y * self.colz.x * self.colw.z
                + self.colx.y * self.colz.z * self.colw.x
                + self.colx.z * self.colz.x * self.colw.y
                - self.colx.z * self.colz.y * self.colw.x,
            -self.colx.x * self.coly.y * self.colw.z
                + self.colx.x * self.coly.z * self.colw.y
                + self.colx.y * self.coly.x * self.colw.z
                - self.colx.y * self.coly.z * self.colw.x
                - self.colx.z * self.coly.x * self.colw.y
                + self.colx.z * self.coly.y * self.colw.x,
            self.colx.x * self.coly.y * self.colz.z
                - self.colx.x * self.coly.z * self.colz.y
                - self.colx.y * self.coly.x * self.colz.z
                + self.colx.y * self.coly.z * self.colz.x
                + self.colx.z * self.coly.x * self.colz.y
                - self.colx.z * self.coly.y * self.colz.x,
        );

        let inv = Self::from_cols(inv_colx, inv_coly, inv_colz, inv_colw);

        let det = inv.colx.x * self.colx.x
            + inv.coly.x * self.coly.x
            + inv.colz.x * self.colz.x
            + inv.colw.x * self.colw.x;

        if det == T::ZERO {
            panic!("try to invert non-invertible matrix");
        }

        inv.div(det)
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
            self.colx.x, self.coly.x, self.colz.x, self.colw.x
        )?;
        writeln!(
            f,
            "[{}, {}, {}, {}]",
            self.colx.y, self.coly.y, self.colz.y, self.colw.y
        )?;
        writeln!(
            f,
            "[{}, {}, {}, {}]",
            self.colx.z, self.coly.z, self.colz.z, self.colw.z
        )?;
        writeln!(
            f,
            "[{}, {}, {}, {}]",
            self.colx.w, self.coly.w, self.colz.w, self.colw.w
        )
    }
}

#[cfg(test)]
mod test {
    use std::{print, println};

    use mini3d_derive::fixed;

    use crate::math::fixed::I32F16;

    use super::*;

    #[test]
    fn test_mat4() {
        let mut v = V4::<I32F16>::new(fixed!(1), fixed!(1), fixed!(0), fixed!(1));
        let t = M4::<I32F16>::rotate_y(I32F16::PI_2);
        let inv_t = t.inverse();
        let (sin, cos) = I32F16::PI_2.sin_cos();
        println!("{}", sin);
        println!("{}", -sin);
        println!("{}", sin.neg());
        println!("{}", t);
        println!("{}", v);
        v = t * v;
        println!("{}", v);
        v = inv_t * v;
        println!("{}", v);
    }
}
