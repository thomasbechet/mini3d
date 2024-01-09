use core::{
    fmt::Display,
    ops::{Add, AddAssign, Div, Mul, Sub},
};

use mini3d_serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};

use crate::fixed::{FixedPoint, FixedPointError, RealFixedPoint, SignedFixedPoint};

use super::{V2, V4};

#[derive(Default, Debug, Copy, Clone)]
pub struct V3<T: FixedPoint> {
    pub x: T,
    pub y: T,
    pub z: T,
}

impl<T: FixedPoint> V3<T> {
    pub const ZERO: Self = Self::new(T::ZERO, T::ZERO, T::ZERO);
    pub const ONE: Self = Self::new(T::ONE, T::ONE, T::ONE);
    pub const X: Self = Self::new(T::ONE, T::ZERO, T::ZERO);
    pub const Y: Self = Self::new(T::ZERO, T::ONE, T::ZERO);
    pub const Z: Self = Self::new(T::ZERO, T::ZERO, T::ONE);

    pub fn cast<F: FixedPoint>(v: V3<F>) -> Self
    where
        <T as FixedPoint>::INNER: TryFrom<<F as FixedPoint>::INNER>,
    {
        Self::new(T::cast(v.x), T::cast(v.y), T::cast(v.z))
    }

    pub fn try_cast<F: FixedPoint>(v: V3<F>) -> Result<Self, FixedPointError>
    where
        <T as FixedPoint>::INNER: TryFrom<<F as FixedPoint>::INNER>,
    {
        Ok(Self::new(
            T::try_cast(v.x)?,
            T::try_cast(v.y)?,
            T::try_cast(v.z)?,
        ))
    }

    pub const fn new(x: T, y: T, z: T) -> Self {
        Self { x, y, z }
    }

    pub const fn from_vec2(v: V2<T>, z: T) -> Self {
        Self::new(v.x, v.y, z)
    }

    pub const fn from_vec4(v: super::V4<T>) -> Self {
        Self::new(v.x, v.y, v.z)
    }

    pub const fn xy(self) -> V2<T> {
        V2::new(self.x, self.y)
    }

    pub fn dot(self, rhs: Self) -> T {
        (self.x * rhs.x) + (self.y * rhs.y) + (self.z * rhs.z)
    }

    pub fn length_squared(self) -> T {
        self.dot(self)
    }

    pub fn cross(self, v: Self) -> Self {
        Self::new(
            self.x * v.y - self.y * v.x,
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
        )
    }

    pub fn project_onto_normalized(self, v: Self) -> Self {
        v * self.dot(v)
    }

    pub fn reject_from_normalized(self, v: Self) -> Self {
        self - self.project_onto_normalized(v)
    }
}

impl<T: FixedPoint + SignedFixedPoint> V3<T> {
    pub const NEG_X: Self = Self::new(T::NEG_ONE, T::ZERO, T::ZERO);
    pub const NEG_Y: Self = Self::new(T::ZERO, T::NEG_ONE, T::ZERO);
    pub const NEG_Z: Self = Self::new(T::ZERO, T::ZERO, T::NEG_ONE);
}

impl<T: FixedPoint + RealFixedPoint> V3<T> {
    pub fn length(self) -> T {
        self.length_squared().sqrt()
    }

    pub fn normalize(self) -> Self {
        self / self.length()
    }

    pub fn normalize_or_zero(self) -> Self {
        let length = self.length();
        if length == T::ZERO {
            Self::ZERO
        } else {
            self / length
        }
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

impl<T: FixedPoint> Add for V3<T> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl<T: FixedPoint> Add<T> for V3<T> {
    type Output = Self;

    fn add(self, rhs: T) -> Self::Output {
        Self::new(self.x + rhs, self.y + rhs, self.z + rhs)
    }
}

impl<T: FixedPoint> AddAssign for V3<T> {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl<T: FixedPoint> Sub for V3<T> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl<T: FixedPoint> Sub<T> for V3<T> {
    type Output = Self;

    fn sub(self, rhs: T) -> Self::Output {
        Self::new(self.x - rhs, self.y - rhs, self.z - rhs)
    }
}

impl<T: FixedPoint> Mul for V3<T> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl<T: FixedPoint> Mul<T> for V3<T> {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        Self::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl<T: FixedPoint> Div for V3<T> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl<T: FixedPoint> Div<T> for V3<T> {
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

impl<T: FixedPoint + Serialize> Serialize for V3<T> {
    type Header = T::Header;

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.x.serialize(encoder)?;
        self.y.serialize(encoder)?;
        self.z.serialize(encoder)?;
        Ok(())
    }

    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        let x = T::deserialize(decoder, header)?;
        let y = T::deserialize(decoder, header)?;
        let z = T::deserialize(decoder, header)?;
        Ok(V3::new(x, y, z))
    }
}

#[cfg(test)]
mod test {
    use std::println;

    use mini3d_derive::fixed;

    use crate::{fixed::I32F24, vec::V3I32F24};

    #[test]
    fn test_vec3() {
        println!("{}", I32F24::EPSILON);
        let x = V3I32F24::from(fixed!(1i32f24));
        println!("{}", x + x / fixed!(0.2123i32f24));
    }
}
