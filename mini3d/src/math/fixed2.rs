use core::ops::{Shl, Shr};

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
                Self((value * (1 << $frac) as f32) as $inner)
            }

            // Reserved for external API call
            pub fn from_f64(value: f64) -> Self {
                Self((value * (1 << $frac) as f64) as $inner)
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

pub trait Fixed {
    const RADIX: u32;
    const SIZE: u32;
    type BITS: Shl<u32, Output = Self::BITS> + Shr<u32, Output = Self::BITS>;
    fn new(bits: Self::BITS) -> Self;
    fn convert<F: Fixed>(self) -> F
    where
        F::BITS: From<Self::BITS>;
}

macro_rules! define_fixed {
    ($name:ident, $base:ty, $scale:expr, $inter:ty, $signed:tt) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name($base);

        impl $name {
            // pub const BITS: u32 = <$base>::BITS;
            // pub const MAX: Self = Self::from_bits(<$base>::MAX);
            // pub const MIN: Self = Self::from_bits(<$base>::MIN);
            // pub const EPSILON: Self = Self::from_bits(1);
            // pub const ZERO: Self = Self::from_bits(0);
            // pub const ONE: Self = Self::from_integer(1);
            // pub const SIGNED: bool = $signed;
            // pub const PI: Self = Self::lit("3.1415926535897932384626433832795028");
            // pub const E: Self = Self::lit("2.7182818284590452353602874713526625");

            // pub const fn lit(lit: &str) -> Self {
            //     let (signed, int, mut frac, dp) = match parse_lit_dec(lit) {
            //         Some(v) => v,
            //         None => panic!("Invalid literal"),
            //     };
            //     let mut fixed = int << $radix;
            //     let base = 10_u64.pow(dp as u32);
            //     let mut i = 0;
            //     while i < $radix {
            //         frac <<= 1; // multiply by 2
            //         if frac >= base {
            //             fixed |= 1 << ($radix - 1 - i);
            //             frac -= base;
            //         }
            //         i += 1;
            //     }
            //     if signed {
            //         fixed = !fixed + 1;
            //     }
            //     Self::from_bits(fixed as $bits)
            // }

            pub const fn from_bits(bits: $base) -> Self {
                Self(bits)
            }

            pub const fn from_integer(value: $base) -> Self {
                Self(((value as $inter) * $scale) as $base)
            }

            pub const fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }

            pub const fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }

            pub const fn mul(self, rhs: Self) -> Self {
                Self(((self.0 as $inter * rhs.0 as $inter) / $scale) as $base)
            }

            pub const fn div(self, rhs: Self) -> Self {
                Self((((self.0 as $inter) * $scale) / rhs.0 as $inter) as $base)
            }

            // pub const fn floor(self) -> Self {
            //     Self(((self.0 as $wide) & !((1 << $radix) - 1)) as $bits)
            // }

            // pub const fn ceil(self) -> Self {
            //     Self(((self.0 as $wide) + ((1 << $radix) - 1) & !((1 << $radix) - 1)) as $bits)
            // }

            pub const fn trunc(self) -> Self {
                Self((self.0 / $scale) * $scale)
            }

            pub const fn frac(self) -> Self {
                Self(self.0 % $scale)
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
                // Convert to decimal base
                let ratio = $scale / 1000;
                let v = (self.0 as $inter) * ratio;
                write!(f, "{}", v)?;
                Ok(())
            }
        }
    };
}

macro_rules! define_unsigned {
    ($name:ident, $bits:ty, $radix:expr, $wide:ty) => {
        define_fixed!($name, $bits, $radix, $wide, false);
    };
}

macro_rules! define_signed {
    ($name:ident, $bits:ty, $radix:expr, $wide:ty) => {
        define_fixed!($name, $bits, $radix, $wide, true);

        impl $name {
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

define_unsigned!(U32F16, u32, (1 << 16), u64);
// define_unsigned!(U32F16, u32, 10, u64);

#[cfg(test)]
mod test {
    use std::println;

    use super::*;

    const fn convert_base(value: u32, base: u32) -> u32 {
        let mut value = value;
        let mut result = 0;
        let mut i = 0;
        while value > 0 {
            let r = value % base;
            value /= base;
            result += r * 10_u32.pow(i);
            i += 1;
        }
        result
    }

    #[test]
    fn test_fixed() {
        // let mut a = U32F16::from_integer(1);
        // println!("{}", a.0);
        // a.0 += 1;
        // println!("{}", a.0);
        // println!("{}", a.trunc().0);

        let b0: u32 = 10; // Base 10
        let e0: u32 = 2; // Exp 2 -> 10^-2 = 0.01
        let b1: u32 = 3; // Base 2
        let e1: u32 = 3; // Exp 2 -> 2^-2 = 0.25

        let a = 123; // Base 10

        // Convert a to base 2
        let mut a1 = a;
        let mut a2 = 0;
        let mut i = 0;
        while a1 > 0 {
            let r = a1 % b1;
            a1 /= b1;
            a2 += r * b0.pow(i);
            i += 1;
        }
        println!("a2: {}", a2);

        // Convert a2 to base 10
        let mut a3 = 0;
        let mut i = 0;
        while a2 > 0 {
            let r = a2 % b0;
            a2 /= b0;
            a3 += r * b1.pow(i);
            i += 1;
        }
        println!("a3: {}", a3);
    }
}
