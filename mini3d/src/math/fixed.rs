use core::{
    fmt::Debug,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Shl, Shr, Sub, SubAssign},
};

use mini3d_derive::Error;

use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};

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
            // Reserved only for external API call
            pub fn from_f32(value: f32) -> Self {
                Self((value * Self::SCALE as f32) as $inner)
            }

            // Reserved only for external API call
            pub fn from_f64(value: f64) -> Self {
                Self((value * Self::SCALE as f64) as $inner)
            }

            // Reserved only for external API call
            pub fn to_f32(self) -> f32 {
                self.0 as f32 / (1 << $frac) as f32
            }

            // Reserved only for external API call
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

pub trait FixedPoint:
    Copy
    + Clone
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + SubAssign
    + Mul<Output = Self>
    + MulAssign
    + Div<Output = Self>
    + DivAssign
    + Eq
    + PartialEq
    + Default
    + Debug
{
    const FRAC: u32;
    const BITS: u32;
    const ZERO: Self;
    const ONE: Self;
    const TWO: Self;
    type INNER: Shl<u32, Output = Self::INNER> + Shr<u32, Output = Self::INNER>;
    fn from_inner(inner: Self::INNER) -> Self;
    fn convert<F: FixedPoint>(self) -> Result<F, FixedPointError>
    where
        F::INNER: TryFrom<Self::INNER>;
    fn powi(self, n: u32) -> Self;
    fn min(self, rhs: Self) -> Self;
    fn max(self, rhs: Self) -> Self;
}

pub trait RealFixedPoint {
    const HALF: Self;
    fn sqrt(self) -> Self;
    fn pow(self, v: Self) -> Self;
    fn recip(self) -> Self;
}

pub trait TrigFixedPoint: Sized + Copy + Clone {
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn tan(self) -> Self;
    fn sin_cos(self) -> (Self, Self);
    fn to_radians(self) -> Self;
    fn to_degrees(self) -> Self;
}

pub trait SignedFixedPoint: Neg<Output = Self> {
    const NEG_ONE: Self;
    fn abs(self) -> Self;
}

macro_rules! define_real {
    ($name:ident, $inner:ty, $inter:ty, $frac:expr, $signed:tt) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name($inner);

        impl FixedPoint for $name {
            const FRAC: u32 = $frac;
            const BITS: u32 = <$inner>::BITS;
            const ZERO: Self = Self::from_int(0);
            const ONE: Self = Self::from_int(1);
            const TWO: Self = Self::from_int(2);
            type INNER = $inner;

            fn from_inner(inner: Self::INNER) -> Self {
                Self(inner)
            }

            fn convert<F: FixedPoint>(self) -> Result<F, FixedPointError>
            where
                F::INNER: TryFrom<Self::INNER>,
            {
                let shift = F::FRAC as isize - Self::FRAC as isize;
                if Self::BITS < F::BITS {
                    let inner = if shift >= 0 {
                        self.0 << (shift as u32)
                    } else if shift < 0 {
                        self.0 >> (-shift as u32)
                    };
                    Ok(F::from_inner(
                        F::INNER::try_from(inner).map_err(|_| FixedPointError::InvalidSign)?,
                    ))
                } else {
                    let inner =
                        F::INNER::try_from(self.0).map_err(|_| FixedPointError::Overflow)?;
                    if shift >= 0 {
                        Ok(F::from_inner(inner << (shift as u32)))
                    } else if shift < 0 {
                        Ok(F::from_inner(inner >> (-shift as u32)))
                    }
                }
            }

            fn powi(self, n: u32) -> Self {
                self.powi(n)
            }

            fn min(self, rhs: Self) -> Self {
                if self.0 < rhs.0 {
                    self
                } else {
                    rhs
                }
            }

            fn max(self, rhs: Self) -> Self {
                if self.0 > rhs.0 {
                    self
                } else {
                    rhs
                }
            }
        }

        impl RealFixedPoint for $name {
            const HALF: Self = Self::from_int(1).div(Self::from_int(2));

            fn sqrt(self) -> Self {
                let mut v = Self::ONE;
                let mut i = 0;
                while i < 10 && ((v * v) - self).abs() >= Self::EPSILON {
                    v = (v + (self / v)) * Self::HALF;
                    i += 1;
                }
                v
            }

            fn pow(self, v: Self) -> Self {
                self.pow(v)
            }

            fn recip(self) -> Self {
                Self::ONE.div(self)
            }
        }

        impl $name {
            pub const SIGNED: bool = $signed;
            pub const BITS: u32 = <$inner>::BITS;
            pub const INT_BITS: u32 = <$inner>::BITS - $frac;
            pub const FRAC_BITS: u32 = $frac;
            pub const SCALE: u64 = 1 << $frac;
            pub const FRAC_MASK: $inner = (1 << $frac) - 1;
            pub const INT_MASK: $inner = !Self::FRAC_MASK;

            pub const MAX: Self = Self::from_inner(<$inner>::MAX);
            pub const MIN: Self = Self::from_inner(<$inner>::MIN);
            pub const EPSILON: Self = Self::from_inner(1);

            pub const HALF: Self = Self::from_int(1).div(Self::from_int(2));
            pub const PI: Self = Self::lit("3.1415926535897932384626433832795028");
            pub const PI_2: Self = Self::lit("1.57079632679489661923132169163975144");
            pub const PI_4: Self = Self::lit("0.785398163397448309615660845819875721");
            pub const E: Self = Self::lit("2.7182818284590452353602874713526625");

            pub const LOGN_DIVIDER_LUT: [Self; 16] = [
                Self::ZERO,           // LogN(1)
                Self::lit("0.6931"),  // LogN(2)
                Self::lit("1.3863"),  // LogN(4)
                Self::lit("2.0794"),  // LogN(8)
                Self::lit("2.7726"),  // LogN(16)
                Self::lit("3.4657"),  // LogN(32)
                Self::lit("4.1589"),  // LogN(64)
                Self::lit("4.8520"),  // LogN(128)
                Self::lit("5.5452"),  // LogN(256)
                Self::lit("6.2383"),  // LogN(512)
                Self::lit("6.9315"),  // LogN(1024)
                Self::lit("7.6246"),  // LogN(2048)
                Self::lit("8.3178"),  // LogN(4096)
                Self::lit("9.0109"),  // LogN(8192)
                Self::lit("9.7041"),  // LogN(16384)
                Self::lit("10.3972"), // LogN(32768)
            ];

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

            pub const fn int(self) -> $inner {
                self.0 / Self::SCALE as $inner
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

            pub const fn powi(self, n: u32) -> Self {
                let mut v = Self::ONE;
                let mut i = 0;
                while i < n {
                    v = v.mul(self);
                    i += 1;
                }
                v
            }

            pub const fn facti(self, n: u32) -> Self {
                let mut v = Self::ONE;
                let mut i = 1;
                while i <= n {
                    v = v.mul(Self::from_int(1));
                    i += 1;
                }
                v
            }

            // Traylor series
            // log(1+x) = x - x^2/2 + x^3/3 - x^4/4 + x^5/5 in (-1, 1)
            // Input range: (0, 2^FRAC-1)
            // log(20) = log(20/16 * 16) = log(25/16) + log(16)
            // log(16) => use LUT
            // log(25/16) => use traylor series (input in 0-2)
            pub const fn ln(mut self: Self) -> Self {
                let mut divider = Self::ONE;

                // Find the divider (v >= divider and v < divider + 1)
                let mut lower = Self::from_int(2);
                let mut exp = 1;
                let mut divider_index = 0;
                while exp < Self::LOGN_DIVIDER_LUT.len() {
                    let upper = lower.mul(Self::from_int(2));
                    if self.0 >= lower.0 && self.0 <= upper.0 {
                        divider = lower;
                        divider_index = exp;
                        break;
                    }
                    lower = upper;
                    exp += 1;
                }

                self = self.div(divider);
                self = self.sub(Self::ONE);

                // Apply traylor series
                let mut result = self;
                result = result.sub(self.powi(2).div(Self::from_int(2)));
                result = result.add(self.powi(3).div(Self::from_int(3)));
                result = result.sub(self.powi(4).div(Self::from_int(4)));
                result = result.add(self.powi(5).div(Self::from_int(5)));
                result = result.sub(self.powi(6).div(Self::from_int(6)));
                result = result.add(self.powi(7).div(Self::from_int(7)));

                // Add LUT value
                result = result.add(Self::LOGN_DIVIDER_LUT[divider_index]);

                result
            }

            // e^x = 1 + x + x^2/2! + x^3/3! + x^4/4! ...
            pub const fn exp(self) -> Self {
                let mut result = Self::ONE;

                let mut iteration = 1;
                while iteration <= 14 {
                    let mut tmp = Self::ONE;
                    let mut i = 1;
                    while i <= iteration {
                        tmp = tmp.mul(self.div(Self::from_int(i)));
                        i += 1;
                    }
                    result = result.add(tmp);
                    iteration += 1;
                }

                result
            }

            // pow(x, y) = e^(y * log(x))
            pub const fn pow(self, v: Self) -> Self {
                v.mul(self.ln()).exp()
            }

            // sin(x) = x - x^3/3! + x^5/5! - x^7/7! ...
            pub const fn sin_taylor(self) -> Self {
                let mut result = self;

                let mut tmp = self.powi(3).div(Self::from_int(6));
                result = result.sub(tmp);

                tmp = self.powi(5).div(Self::from_int(120));
                result = result.add(tmp);

                // TODO: fix for 8bits integer part
                let fac7 = Self::from_int(5040);
                tmp = self.powi(7).div(fac7);
                result = result.sub(tmp);

                tmp = self.powi(9).div(fac7);
                tmp = tmp.div(Self::from_int(9 * 8));
                result = result.add(tmp);

                result
            }

            // cos(x) = x - x^2/2! + x^4/4! - x^6/6! + x^8/8! ...
            pub const fn cos_taylor(self) -> Self {
                let mut result = Self::ONE;

                let mut tmp = self.powi(2).div(Self::from_int(2));
                result = result.sub(tmp);

                tmp = self.powi(4).div(Self::from_int(24));
                result = result.add(tmp);

                tmp = self.powi(6).div(Self::from_int(720));
                result = result.sub(tmp);

                tmp = self.powi(8).div(Self::from_int(5040));
                tmp = tmp.div(Self::from_int(8));
                result = result.add(tmp);

                result
            }

            pub const fn tan_taylor(self) -> Self {
                self.sin_taylor().div(self.cos_taylor())
            }

            // Input in [0, PI/2]
            pub const fn sin_cos(self) -> (Self, Self) {
                // minimax polynomial approximation for sine on [0, PI/4]
                let s0: Self = Self::lit("0.00019510998390614986");
                let s1: Self = Self::lit("0.0083322080317884684");
                let s2: Self = Self::lit("0.16666648373939097");
                let s3: Self = Self::lit("0.99999991734512150");
                // minimax polynomial approximation for cosine on [0, PI/4]
                let c0: Self = Self::lit("0.0013578890357166529");
                let c1: Self = Self::lit("0.041654359549283981");
                let c2: Self = Self::lit("0.49999838648363948");
                let c3: Self = Self::lit("0.99999997159466147");

                // reduce range from [0, PI/2] to [0, PI/4]
                let t = if self.0 > Self::PI_4.0 {
                    Self::PI_2.sub(self)
                } else {
                    self
                };

                // scale up argument for maximum precision
                // let a = t.mul(Self::from_int(1 << Self::FRAC_BITS));
                let a = t;

                // pre-compute a^2 and a^4
                let s = a.mul(a);
                let q = s.mul(s);

                // approximate sine on [0, PI/4]
                let h = s3.sub(s2.mul(s));
                let l = s1.sub(s0.mul(s)).mul(q);
                let sn = h.add(l).mul(a);

                // approximate cos on [0, PI/4]
                let h = c3.sub(c2.mul(s));
                let l = c1.sub(c0.mul(s)).mul(q);
                let cs = h.add(l);

                // round result to output precision

                let sin = if t.0 != self.0 { cs } else { sn };
                let cos = if t.0 != self.0 { sn } else { cs };

                (sin, cos)
            }
        }

        impl TrigFixedPoint for $name {
            fn sin(self) -> Self {
                self.sin_taylor()
            }

            fn cos(self) -> Self {
                self.cos_taylor()
            }

            fn tan(self) -> Self {
                self.tan_taylor()
            }

            fn sin_cos(self) -> (Self, Self) {
                self.sin_cos()
            }

            fn to_radians(self) -> Self {
                self.mul(Self::PI).div(Self::from_int(180))
            }

            fn to_degrees(self) -> Self {
                self.mul(Self::from_int(180)).div(Self::PI)
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl From<&str> for $name {
            fn from(lit: &str) -> Self {
                Self::lit(lit)
            }
        }

        // pub fn from<F: FixedPoint>(value: F) -> Self
        // where
        //     <$name as FixedPoint>::INNER: TryFrom<<F as FixedPoint>::INNER>,
        // {
        //     value.convert::<$name>().unwrap()
        // }

        // pub fn into<F: FixedPoint>(self) -> F
        // where
        //     <F as FixedPoint>::INNER: TryFrom<<$name as FixedPoint>::INNER>,
        // {
        //     self.convert::<F>().unwrap()
        // }

        impl<S, F: FixedPoint<INNER = S>> From<F> for $name
        where
            <$name as FixedPoint>::INNER: TryFrom<<F as FixedPoint>::INNER>,
        {
            fn from(value: F) -> Self {
                value.convert::<$name>().unwrap()
            }
        }

        // impl<F: FixedPoint> Into<F> for $name
        // where
        //     <F as FixedPoint>::INNER: TryFrom<<$name as FixedPoint>::INNER>,
        // {
        //     fn into(self) -> F {
        //         self.convert::<F>().unwrap()
        //     }
        // }

        // impl From<u8> for $name {
        //     fn from(value: u8) -> Self {
        //         Self::from_int(value as $inner)
        //     }
        // }

        // impl From<u16> for $name {
        //     fn from(value: u16) -> Self {
        //         Self::from_int(value as $inner)
        //     }
        // }

        // impl From<u32> for $name {
        //     fn from(value: u32) -> Self {
        //         Self::from_int(value as $inner)
        //     }
        // }

        // impl From<u64> for $name {
        //     fn from(value: u64) -> Self {
        //         Self::from_int(value as $inner)
        //     }
        // }

        impl Add for $name {
            type Output = Self;

            fn add(self, rhs: Self) -> Self::Output {
                self.add(rhs)
            }
        }

        impl Add<&$name> for $name {
            type Output = $name;

            fn add(self, rhs: &Self) -> Self::Output {
                self.add(*rhs)
            }
        }

        impl AddAssign for $name {
            fn add_assign(&mut self, rhs: Self) {
                *self = self.add(rhs);
            }
        }

        impl Add<u32> for $name {
            type Output = Self;

            fn add(self, rhs: u32) -> Self::Output {
                self.add(Self::from_int(rhs as $inner))
            }
        }

        impl Add<$name> for u32 {
            type Output = $name;

            fn add(self, rhs: $name) -> Self::Output {
                $name::from_int(self as $inner).add(rhs)
            }
        }

        impl Sub for $name {
            type Output = Self;

            fn sub(self, rhs: Self) -> Self::Output {
                self.sub(rhs)
            }
        }

        impl Sub<&$name> for $name {
            type Output = $name;

            fn sub(self, rhs: &Self) -> Self::Output {
                self.sub(*rhs)
            }
        }

        impl SubAssign for $name {
            fn sub_assign(&mut self, rhs: Self) {
                *self = self.sub(rhs);
            }
        }

        impl Sub<u32> for $name {
            type Output = Self;

            fn sub(self, rhs: u32) -> Self::Output {
                self.sub(Self::from_int(rhs as $inner))
            }
        }

        impl Sub<$name> for u32 {
            type Output = $name;

            fn sub(self, rhs: $name) -> Self::Output {
                $name::from_int(self as $inner).sub(rhs)
            }
        }

        impl Mul for $name {
            type Output = Self;

            fn mul(self, rhs: Self) -> Self::Output {
                self.mul(rhs)
            }
        }

        impl Mul<&$name> for $name {
            type Output = $name;

            fn mul(self, rhs: &Self) -> Self::Output {
                self.mul(*rhs)
            }
        }

        impl MulAssign for $name {
            fn mul_assign(&mut self, rhs: Self) {
                *self = self.mul(rhs);
            }
        }

        impl Mul<u32> for $name {
            type Output = Self;

            fn mul(self, rhs: u32) -> Self::Output {
                self.mul(Self::from_int(rhs as $inner))
            }
        }

        impl Mul<$name> for u32 {
            type Output = $name;

            fn mul(self, rhs: $name) -> Self::Output {
                $name::from_int(self as $inner).mul(rhs)
            }
        }

        impl Div for $name {
            type Output = Self;

            fn div(self, rhs: Self) -> Self::Output {
                self.div(rhs)
            }
        }

        impl Div<&$name> for $name {
            type Output = $name;

            fn div(self, rhs: &Self) -> Self::Output {
                self.div(*rhs)
            }
        }

        impl DivAssign for $name {
            fn div_assign(&mut self, rhs: Self) {
                *self = self.div(rhs);
            }
        }

        impl Div<u32> for $name {
            type Output = Self;

            fn div(self, rhs: u32) -> Self::Output {
                self.div(Self::from_int(rhs as $inner))
            }
        }

        impl Div<$name> for u32 {
            type Output = $name;

            fn div(self, rhs: $name) -> Self::Output {
                $name::from_int(self as $inner).div(rhs)
            }
        }

        impl core::fmt::Display for $name {
            #[allow(unused_comparisons)]
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                // Print integer part
                if self.0 < 0 {
                    write!(f, "-")?;
                }
                write!(f, "{}.", self.abs().floor().0 as u64 >> $frac)?;
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

        impl Serialize for $name {
            type Header = <$inner as Serialize>::Header;

            fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
                self.0.serialize(encoder)
            }

            fn deserialize(
                decoder: &mut impl Decoder,
                header: &Self::Header,
            ) -> Result<Self, DecoderError> {
                Ok(Self::from_inner(<$inner>::deserialize(decoder, header)?))
            }
        }
    };
}

macro_rules! define_real_unsigned {
    ($name:ident, $inner:ty, $inter:ty, $frac:expr) => {
        define_real!($name, $inner, $inter, $frac, false);

        impl $name {
            pub const fn abs(self) -> Self {
                self
            }
        }
    };
}

macro_rules! define_real_signed {
    ($name:ident, $inner:ty, $inter:ty, $frac:expr) => {
        define_real!($name, $inner, $inter, $frac, true);

        impl $name {
            pub const NEG_ONE: Self = Self::from_int(-1);

            pub const fn abs(self) -> Self {
                Self(self.0.abs())
            }

            pub const fn neg(self) -> Self {
                Self(-self.0)
            }
        }

        impl Neg for $name {
            type Output = Self;

            fn neg(self) -> Self::Output {
                self.neg()
            }
        }

        impl SignedFixedPoint for $name {
            const NEG_ONE: Self = Self::NEG_ONE;

            fn abs(self) -> Self {
                self.abs()
            }
        }

        // impl From<i16> for $name {
        //     fn from(value: i16) -> Self {
        //         Self::from_int(value as $inner)
        //     }
        // }

        // impl From<i32> for $name {
        //     fn from(value: i32) -> Self {
        //         Self::from_int(value as $inner)
        //     }
        // }

        // impl From<i64> for $name {
        //     fn from(value: i64) -> Self {
        //         Self::from_int(value as $inner)
        //     }
        // }
    };
}

macro_rules! define_num_unsigned {
    ($inner:ty, $inter:ty) => {
        impl FixedPoint for $inner {
            const FRAC: u32 = 0;
            const BITS: u32 = Self::BITS;
            const ZERO: Self = 0;
            const ONE: Self = 1;
            const TWO: Self = 2;
            type INNER = Self;

            fn from_inner(inner: Self::INNER) -> Self {
                inner
            }

            fn convert<F: FixedPoint>(self) -> Result<F, FixedPointError>
            where
                F::INNER: TryFrom<Self::INNER>,
            {
                let shift = F::FRAC as isize - Self::FRAC as isize;
                if Self::BITS < F::BITS {
                    let inner = if shift > 0 {
                        self << (shift as u32)
                    } else if shift < 0 {
                        self >> (-shift as u32)
                    } else {
                        self
                    };
                    Ok(F::from_inner(
                        F::INNER::try_from(inner).map_err(|_| FixedPointError::InvalidSign)?,
                    ))
                } else {
                    let inner = F::INNER::try_from(self).map_err(|_| FixedPointError::Overflow)?;
                    if shift > 0 {
                        Ok(F::from_inner(inner << (shift as u32)))
                    } else if shift < 0 {
                        Ok(F::from_inner(inner >> (-shift as u32)))
                    } else {
                        Ok(F::from_inner(inner))
                    }
                }
            }

            fn powi(self, n: u32) -> Self {
                self.powi(n)
            }

            fn min(self, rhs: Self) -> Self {
                if self < rhs {
                    self
                } else {
                    rhs
                }
            }

            fn max(self, rhs: Self) -> Self {
                if self > rhs {
                    self
                } else {
                    rhs
                }
            }
        }
    };
}

macro_rules! define_num_signed {
    ($inner:ty, $inter:ty) => {
        define_num_unsigned!($inner, $inter);
        impl SignedFixedPoint for $inner {
            const NEG_ONE: Self = -1;

            fn abs(self) -> Self {
                self.abs()
            }
        }
    };
}

define_real_unsigned!(U64F16, u64, u128, 16);
define_real_signed!(I64F16, i64, i128, 16);
define_real_unsigned!(U64F32, u64, u128, 32);
define_real_signed!(I64F32, i64, i128, 32);

define_real_unsigned!(U32F8, u32, u64, 8);
impl_float_conversion!(U32F8, u32, 8);
define_real_signed!(I32F8, i32, i64, 8);
impl_float_conversion!(I32F8, i32, 8);
define_real_signed!(I32F16, i32, i64, 16);
impl_float_conversion!(I32F16, i32, 16);
define_real_unsigned!(U32F16, u32, u64, 16);
impl_float_conversion!(U32F16, u32, 16);
define_real_unsigned!(U32F24, u32, u64, 24);
impl_float_conversion!(U32F24, u32, 24);
define_real_signed!(I32F24, i32, i64, 24);
impl_float_conversion!(I32F24, i32, 24);

define_real_unsigned!(U16F8, u16, u32, 8);
impl_float_conversion!(U16F8, u16, 8);
define_real_signed!(I16F8, i16, i32, 8);
impl_float_conversion!(I16F8, i16, 8);

define_num_unsigned!(u16, u32);
define_num_signed!(i16, i32);
define_num_unsigned!(u32, u64);
define_num_signed!(i32, i64);
define_num_unsigned!(u64, u128);
define_num_signed!(i64, i128);

#[cfg(test)]
mod test {
    use std::println;

    use mini3d_derive::fixed;

    use super::*;

    #[test]
    fn test_unsigned() {
        assert_eq!(U32F16::lit("1.234").floor(), U32F16::ONE);
        assert_eq!(U32F16::lit("1.234").ceil(), fixed!(2));
        assert_eq!(U32F16::lit("1.2").round(), U32F16::ONE);
        assert_eq!(U32F16::lit("1.5").round(), fixed!(2));
    }

    #[test]
    fn test_signed() {
        assert_eq!(I32F16::lit("1.2").floor(), I32F16::ONE);
        assert_eq!(I32F16::lit("-1.2").floor(), fixed!(-2i32f16));
        assert_eq!(I32F16::lit("-1.2").ceil(), I32F16::NEG_ONE);
        assert_eq!(I32F16::lit("-1.2").round(), I32F16::NEG_ONE);
        assert_eq!(I32F16::lit("-0.4").round(), I32F16::ZERO);
        assert_eq!(I32F16::lit("-1.4").abs(), I32F16::lit("1.4"));
        assert_eq!(I32F16::lit("-1.4").neg(), I32F16::lit("1.4"));
    }

    #[test]
    fn test_f() {
        // let x = I32F16::PI_2;
        let x = I32F16::lit("0.999984741210937");
        println!("x {}", x);
        println!("-x {}", -x);
        println!("x {}", x);
        let (sin, _) = x.sin_cos();
        println!("sin {}", sin);
        println!("-sin {}", -sin);
    }
}
