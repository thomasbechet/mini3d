use core::{
    fmt::Debug,
    ops::{Shl, Shr},
};

use mini3d_derive::Error;

const fn parse_lit_dec(float: &str) -> Option<(bool, u64, u64, u8)> {
    let mut signed = false;
    let mut int = 0u64;
    let mut frac = 0u64;
    let mut dp = 0u8;
    let mut frac_flag = false;
    let chars = float.as_bytes();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            b'-' => {
                if i != 0 {
                    return None;
                }
                signed = true;
            }
            b'0'..=b'9' => {
                if frac_flag {
                    // Prevent overflow of pow function during conversion of decimal part
                    // to binary fixed point.
                    if dp >= (u64::MAX.ilog10() - 1) as u8 {
                        break;
                    }
                    if let Some(mul) = frac.checked_mul(10) {
                        frac = mul + (c as u64 - '0' as u64);
                        dp += 1;
                    } else {
                        break;
                    }
                } else if let Some(mul) = int.checked_mul(10) {
                    int = mul + (c as u64 - '0' as u64);
                } else {
                    break;
                }
            }
            b'.' => {
                if frac_flag {
                    return None;
                }
                frac_flag = true;
            }
            _ => return None,
        }
        i += 1;
    }
    Some((signed, int, frac, dp))
}

macro_rules! impl_float_conversion {
    ($name:ident, $inner:ty, $frac:expr) => {
        impl $name {
            // Reserved for external API call
            pub fn from_f32(value: f32) -> Self {
                Self((value * Self::SCALE as f32) as $inner)
            }

            // Reserved for external API call
            pub fn from_f64(value: f64) -> Self {
                Self((value * Self::SCALE as f64) as $inner)
            }

            // Reserved for external API call
            pub fn to_f32(self) -> f32 {
                self.0 as f32 / (1 << $frac) as f32
            }

            // Reserved for external API call
            pub fn to_f64(self) -> f64 {
                self.0 as f64 / (1 << $frac) as f64
            }
        }
    };
}

#[derive(Error, Debug)]
pub enum FixedPointError {
    #[error("invalid sign")]
    InvalidSign,
    #[error("overflow")]
    Overflow,
}

pub trait FixedPoint {
    const FRAC: u32;
    const BITS: u32;
    type INNER: Shl<u32, Output = Self::INNER> + Shr<u32, Output = Self::INNER>;
    fn new(inner: Self::INNER) -> Self;
    fn convert<F: FixedPoint>(self) -> Result<F, FixedPointError>
    where
        F::INNER: TryFrom<Self::INNER>;
}

macro_rules! define_fixed {
    ($name:ident, $inner:ty, $inter:ty, $frac:expr, $signed:tt) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name($inner);

        impl FixedPoint for $name {
            const FRAC: u32 = $frac;
            const BITS: u32 = <$inner>::BITS;
            type INNER = $inner;

            fn new(inner: Self::INNER) -> Self {
                Self(inner)
            }

            fn convert<F: FixedPoint>(self) -> Result<F, FixedPointError>
            where
                F::INNER: TryFrom<Self::INNER>,
            {
                let shift = F::FRAC as isize - Self::FRAC as isize;
                if Self::BITS < F::BITS {
                    let inner = if shift > 0 {
                        self.0 << (shift as u32)
                    } else if shift < 0 {
                        self.0 >> (-shift as u32)
                    } else {
                        self.0
                    };
                    Ok(F::new(
                        F::INNER::try_from(inner).map_err(|_| FixedPointError::InvalidSign)?,
                    ))
                } else {
                    let inner =
                        F::INNER::try_from(self.0).map_err(|_| FixedPointError::Overflow)?;
                    if shift > 0 {
                        Ok(F::new(inner << (shift as u32)))
                    } else if shift < 0 {
                        Ok(F::new(inner >> (-shift as u32)))
                    } else {
                        Ok(F::new(inner))
                    }
                }
            }
        }

        impl $name {
            pub const SIGNED: bool = $signed;
            pub const BITS: u32 = <$inner>::BITS;
            pub const INT_BITS: u32 = <$inner>::BITS - $frac;
            pub const FRAC_BITS: u32 = $frac;
            pub const SCALE: u64 = 1 << $frac;
            pub const FRAC_MASK: $inner = (1 << $frac) - 1;

            pub const MAX: Self = Self::from_inner(<$inner>::MAX);
            pub const MIN: Self = Self::from_inner(<$inner>::MIN);
            pub const EPSILON: Self = Self::from_inner(1);
            pub const ZERO: Self = Self::from_inner(0);
            pub const ONE: Self = Self::from_int(1);
            pub const TWO: Self = Self::from_int(2);
            pub const HALF: Self = Self(1 << ($frac - 1) as $inner);
            pub const PI: Self = Self::lit("3.1415926535897932384626433832795028");
            pub const E: Self = Self::lit("2.7182818284590452353602874713526625");

            pub fn from<F: FixedPoint>(value: F) -> Self
            where
                <$name as FixedPoint>::INNER: TryFrom<<F as FixedPoint>::INNER>,
            {
                value.convert::<$name>().unwrap()
            }

            pub fn into<F: FixedPoint>(self) -> F
            where
                <F as FixedPoint>::INNER: TryFrom<<$name as FixedPoint>::INNER>,
            {
                self.convert::<F>().unwrap()
            }

            pub const fn lit(lit: &str) -> Self {
                let (signed, int, mut frac, dp) = match parse_lit_dec(lit) {
                    Some(v) => v,
                    None => panic!("invalid literal"),
                };
                if signed && !Self::SIGNED {
                    panic!("invalid literal sign");
                }
                let mut fixed = int << $frac;
                let base = 10_u64.pow(dp as u32);
                let mut i = 0;
                while i < $frac {
                    frac <<= 1; // multiply by 2
                    if frac >= base {
                        fixed |= 1 << ($frac - 1 - i);
                        frac -= base;
                    }
                    i += 1;
                }
                if signed {
                    fixed = !fixed + 1;
                }
                Self::from_inner(fixed as $inner)
            }

            pub const fn from_inner(inner: $inner) -> Self {
                Self(inner)
            }

            pub const fn from_int(value: $inner) -> Self {
                Self(value * Self::SCALE as $inner)
            }

            pub const fn try_from_int(value: $inner) -> Option<Self> {
                match value.checked_mul(Self::SCALE as $inner) {
                    Some(v) => Some(Self(v)),
                    None => None,
                }
            }

            pub const fn checked_add(self, rhs: Self) -> Option<Self> {
                match self.0.checked_add(rhs.0) {
                    Some(v) => Some(Self(v)),
                    None => None,
                }
            }

            pub const fn checked_sub(self, rhs: Self) -> Option<Self> {
                match self.0.checked_sub(rhs.0) {
                    Some(v) => Some(Self(v)),
                    None => None,
                }
            }

            pub const fn checked_mul(self, rhs: Self) -> Option<Self> {
                let v = (self.0 as $inter * rhs.0 as $inter) >> $frac;
                if v > <$inner>::MAX as $inter {
                    None
                } else {
                    Some(Self(v as $inner))
                }
            }

            pub const fn checked_div(self, rhs: Self) -> Option<Self> {
                let v = ((self.0 as $inter) << $frac) / rhs.0 as $inter;
                if v > <$inner>::MAX as $inter {
                    None
                } else {
                    Some(Self(v as $inner))
                }
            }

            pub const fn add(self, rhs: Self) -> Self {
                if let Some(v) = self.checked_add(rhs) {
                    v
                } else {
                    panic!("fixed-point add overflow");
                }
            }

            pub const fn sub(self, rhs: Self) -> Self {
                if let Some(v) = self.checked_sub(rhs) {
                    v
                } else {
                    panic!("fixed-point sub overflow");
                }
            }

            pub const fn mul(self, rhs: Self) -> Self {
                if let Some(v) = self.checked_mul(rhs) {
                    v
                } else {
                    panic!("fixed-point mul overflow");
                }
            }

            pub const fn div(self, rhs: Self) -> Self {
                if let Some(v) = self.checked_div(rhs) {
                    v
                } else {
                    panic!("fixed-point div overflow");
                }
            }

            pub const fn trunc(self) -> Self {
                Self((self.0 >> $frac) << $frac)
            }

            pub const fn frac(self) -> Self {
                Self(self.0 & Self::FRAC_MASK)
            }

            pub const fn floor(self) -> Self {
                Self(self.0 & !Self::FRAC_MASK)
            }

            pub const fn ceil(self) -> Self {
                Self(self.0 + Self::FRAC_MASK & !Self::FRAC_MASK)
            }

            pub const fn round(self) -> Self {
                (self.add(Self::HALF)).floor()
            }

            pub const fn int(self) -> $inner {
                self.0 / Self::SCALE as $inner
            }
        }

        impl core::ops::Add for $name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                self.add(rhs)
            }
        }

        impl core::ops::Sub for $name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                self.sub(rhs)
            }
        }

        impl core::ops::Mul for $name {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                self.mul(rhs)
            }
        }

        impl core::ops::Div for $name {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                self.div(rhs)
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                // Print integer part
                write!(f, "{}.", self.int())?;
                // Print fractional part
                let mut frac = self.abs().0 & Self::FRAC_MASK;
                if frac == 0 {
                    write!(f, "0")?;
                }
                while frac > 0 {
                    frac = frac.wrapping_mul(10);
                    write!(f, "{}", frac >> $frac)?;
                    frac &= Self::FRAC_MASK;
                }
                Ok(())
            }
        }
    };
}

macro_rules! define_unsigned {
    ($name:ident, $inner:ty, $inter:ty, $frac:expr) => {
        define_fixed!($name, $inner, $inter, $frac, false);

        impl $name {
            pub const fn abs(self) -> Self {
                self
            }
        }
    };
}

macro_rules! define_signed {
    ($name:ident, $inner:ty, $inter:ty, $frac:expr) => {
        define_fixed!($name, $inner, $inter, $frac, true);

        impl $name {
            pub const MINUS_ONE: Self = Self::from_int(-1);
            pub const MINUS_TWO: Self = Self::from_int(-2);

            pub const fn abs(self) -> Self {
                Self(self.0.abs())
            }

            pub const fn neg(self) -> Self {
                Self(-self.0)
            }
        }

        impl core::ops::Neg for $name {
            type Output = Self;

            fn neg(self) -> Self::Output {
                self.neg()
            }
        }
    };
}

define_unsigned!(U64F32, u64, u128, 32);
define_signed!(I64F32, i64, i128, 32);
define_unsigned!(U64F16, u64, u128, 16);
define_signed!(I64F16, i64, i128, 16);

define_unsigned!(U32F24, u32, u64, 24);
impl_float_conversion!(U32F24, u32, 24);
define_signed!(I32F24, i32, i64, 24);
impl_float_conversion!(I32F24, i32, 24);
define_signed!(I32F16, i32, i64, 16);
impl_float_conversion!(I32F16, i32, 16);
define_unsigned!(U32F16, u32, u64, 16);
impl_float_conversion!(U32F16, u32, 16);
define_unsigned!(U32F8, u32, u64, 8);
impl_float_conversion!(U32F8, u32, 8);
define_signed!(I32F8, i32, i64, 8);
impl_float_conversion!(I32F8, i32, 8);

define_unsigned!(U16F8, u16, u32, 8);
impl_float_conversion!(U16F8, u16, 8);
define_signed!(I16F8, i16, i32, 8);
impl_float_conversion!(I16F8, i16, 8);

#[cfg(test)]
mod test {
    use std::println;

    use mini3d_derive::fixed;

    use super::*;

    #[test]
    fn test_unsigned() {
        assert_eq!(U32F16::lit("1.234").int(), 1);
        assert_eq!(U32F16::lit("1.234").floor(), U32F16::ONE);
        assert_eq!(U32F16::lit("1.234").ceil(), U32F16::TWO);
        assert_eq!(U32F16::lit("1.2").round(), U32F16::ONE);
        assert_eq!(U32F16::lit("1.5").round(), U32F16::TWO);
    }

    #[test]
    fn test_signed() {
        assert_eq!(I32F16::lit("-1").int(), -1);
        assert_eq!(I32F16::lit("1.2").floor(), I32F16::ONE);
        assert_eq!(I32F16::lit("-1.2").floor(), I32F16::MINUS_TWO);
        assert_eq!(I32F16::lit("-1.2").ceil(), I32F16::MINUS_ONE);
        assert_eq!(I32F16::lit("-1.2").round(), I32F16::MINUS_ONE);
        assert_eq!(I32F16::lit("-0.4").round(), I32F16::ZERO);
        assert_eq!(I32F16::lit("-1.4").abs(), I32F16::lit("1.4"));
        assert_eq!(I32F16::lit("-1.4").neg(), I32F16::lit("1.4"));
    }

    #[test]
    fn test_fixed() {
        let x = I32F24::from_f32(1.123123);
        println!("{}", x);
        let x: U32F16 = x.into();
        let x = fixed!(-123.2i32f24 + 1.22i32f16.into());
        println!("{}", x);
    }
}
