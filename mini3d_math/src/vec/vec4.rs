use core::{
    fmt::Display,
    ops::{Add, Div, Mul, Sub},
};

use mini3d_serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};

use crate::fixed::{FixedPoint, FixedPointError, RealFixedPoint, SignedFixedPoint};

use super::{V2, V3};

#[derive(Default, Debug, Copy, Clone)]
pub struct V4<T: FixedPoint> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl<T: FixedPoint> V4<T> {
    pub const ZERO: Self = Self::new(T::ZERO, T::ZERO, T::ZERO, T::ZERO);
    pub const X: Self = Self::new(T::ONE, T::ZERO, T::ZERO, T::ZERO);
    pub const Y: Self = Self::new(T::ZERO, T::ONE, T::ZERO, T::ZERO);
    pub const Z: Self = Self::new(T::ZERO, T::ZERO, T::ONE, T::ZERO);
    pub const W: Self = Self::new(T::ZERO, T::ZERO, T::ZERO, T::ONE);

    pub fn cast<F: FixedPoint>(v: V4<F>) -> Self
    where
        <T as FixedPoint>::INNER: TryFrom<<F as FixedPoint>::INNER>,
    {
        Self::new(T::cast(v.x), T::cast(v.y), T::cast(v.z), T::cast(v.w))
    }

    pub fn try_cast<F: FixedPoint>(v: V4<F>) -> Result<Self, FixedPointError>
    where
        <T as FixedPoint>::INNER: TryFrom<<F as FixedPoint>::INNER>,
    {
        Ok(Self::new(
            T::try_cast(v.x)?,
            T::try_cast(v.y)?,
            T::try_cast(v.z)?,
            T::try_cast(v.w)?,
        ))
    }

    pub const fn new(x: T, y: T, z: T, w: T) -> Self {
        Self { x, y, z, w }
    }

    pub const fn from_vec2(v: V2<T>, z: T, w: T) -> Self {
        Self::new(v.x, v.y, z, w)
    }

    pub const fn from_vec3(v: V3<T>, w: T) -> Self {
        Self::new(v.x, v.y, v.z, w)
    }

    pub const fn xy(self) -> V2<T> {
        V2::new(self.x, self.y)
    }

    pub const fn xyz(self) -> V3<T> {
        V3::new(self.x, self.y, self.z)
    }

    pub fn dot(self, v: Self) -> T {
        (self.x * v.x) + (self.y * v.y) + (self.z * v.z) + (self.w * v.w)
    }

    pub fn length_squared(self) -> T {
        self.dot(self)
    }
}

impl<T: FixedPoint + SignedFixedPoint> V4<T> {
    pub const NEG_X: Self = Self::new(T::NEG_ONE, T::ZERO, T::ZERO, T::ZERO);
    pub const NEG_Y: Self = Self::new(T::ZERO, T::NEG_ONE, T::ZERO, T::ZERO);
    pub const NEG_Z: Self = Self::new(T::ZERO, T::ZERO, T::NEG_ONE, T::ZERO);
    pub const NEG_W: Self = Self::new(T::ZERO, T::ZERO, T::ZERO, T::NEG_ONE);
}

impl<T: FixedPoint + RealFixedPoint> V4<T> {
    pub fn length(self) -> T {
        self.length_squared().sqrt()
    }

    pub fn normalize(self) -> Self {
        self / self.length()
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

impl<T: FixedPoint> Add for V4<T> {
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

impl<T: FixedPoint> Add<T> for V4<T> {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs, self.z + rhs, self.w + rhs)
    }
}

impl<T: FixedPoint> Sub for V4<T> {
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

impl<T: FixedPoint> Sub<T> for V4<T> {
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self::new(self.x - rhs, self.y - rhs, self.z - rhs, self.w - rhs)
    }
}

impl<T: FixedPoint> Mul for V4<T> {
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

impl<T: FixedPoint> Mul<T> for V4<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs, self.w * rhs)
    }
}

impl<T: FixedPoint> Div for V4<T> {
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

impl<T: FixedPoint> Div<T> for V4<T> {
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

impl<T: FixedPoint + Serialize> Serialize for V4<T> {
    type Header = T::Header;

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.x.serialize(encoder)?;
        self.y.serialize(encoder)?;
        self.z.serialize(encoder)?;
        self.w.serialize(encoder)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let x = T::deserialize(decoder, header)?;
        let y = T::deserialize(decoder, header)?;
        let z = T::deserialize(decoder, header)?;
        let w = T::deserialize(decoder, header)?;
        Ok(V4::new(x, y, z, w))
    }
}

#[cfg(test)]
mod test {
    use std::println;

    use mini3d_derive::fixed;

    use crate::{fixed::I32F24, vec::V4I32F24};

    #[test]
    fn test_vec4() {
        println!("{}", I32F24::EPSILON);
        let x = V4I32F24::from(fixed!(1i32f24));
        println!("{}", x + x / fixed!(0.2123i32f24));
    }
}
