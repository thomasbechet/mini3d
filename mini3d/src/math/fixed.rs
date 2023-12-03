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

// pub struct Fixed<B: Bits, const R: u32>(B);

// impl<B: Bits, const R: u32> From<f32> for Fixed<B, R> {
//     fn from(value: f32) -> Self {
//         Self(B::from_bits((value * (1u64.wrapping_shl(R)) as f32) as u64))
//     }
// }

// const fn fmask<const R: u32>(bits: u32) -> u64 {
//     if R == 0 {
//         0
//     } else {
//         let full = !0;
//         full >> (bits - R)
//     }
// }

// impl<B: Bits, const R: u32> Fixed<B, R> {
//     pub const EPSILON: Self = Self(B::ONE);

//     pub const fn convert<B2: Bits, const R2: u32>(self) -> Fixed<B2, R2>
//     where
//         B2: From<B>,
//     {
//         let shift = R2 as isize - R as isize;
//         let b2 = if B::BITS > B2::BITS {
//             let v = if shift > 0 {
//                 self.0 << (shift as u32)
//             } else if shift < 0 {
//                 self.0 >> (-shift as u32)
//             } else {
//                 self.0
//             };
//             B2::from(v)
//         } else {
//             let b2 = B2::from(self.0);
//             if shift > 0 {
//                 b2 << (shift as u32)
//             } else if shift < 0 {
//                 b2 >> (-shift as u32)
//             } else {
//                 b2
//             }
//         };
//         Fixed(b2)
//     }
// }

// impl<B: Bits + core::fmt::Display, const R: u32> core::fmt::Display for Fixed<B, R> {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         let v = self.0.to_bits();
//         let fmask = fmask::<R>(B::BITS);
//         // Print integer part
//         write!(f, "{}.", v >> R)?;
//         // Print fractional part
//         let mut frac = v & fmask;
//         while frac > 0 {
//             frac = frac.wrapping_mul(10);
//             write!(f, "{}", frac >> R)?;
//             frac &= fmask;
//         }
//         Ok(())
//     }
// }

pub trait Fixed {
    const RADIX: u32;
    type BITS;
    fn convert<F: Fixed>(self) -> F;
}

macro_rules! define_fixed {
    ($name:ident, $bits:ty, $radix:expr, $wide:ty, $signed:tt) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub struct $name($bits);

        impl $name {
            pub const BITS: u32 = <$bits>::BITS;
            pub const MAX: Self = Self::from_bits(<$bits>::MAX);
            pub const MIN: Self = Self::from_bits(<$bits>::MIN);
            pub const EPSILON: Self = Self::from_bits(1);
            pub const ZERO: Self = Self::from_bits(0);
            pub const ONE: Self = Self::from_integer(1);
            pub const SIGNED: bool = $signed;
            pub const PI: Self = Self::lit("3.1415926535897932384626433832795028");
            pub const E: Self = Self::lit("2.7182818284590452353602874713526625");

            pub const fn lit(lit: &str) -> Self {
                let (signed, int, mut frac, dp) = match parse_lit_dec(lit) {
                    Some(v) => v,
                    None => panic!("Invalid literal"),
                };
                let mut fixed = int << $radix;
                let base = 10_u64.pow(dp as u32);
                let mut i = 0;
                while i < $radix {
                    frac <<= 1; // multiply by 2
                    if frac >= base {
                        fixed |= 1 << ($radix - 1 - i);
                        frac -= base;
                    }
                    i += 1;
                }
                if signed {
                    fixed = !fixed + 1;
                }
                Self::from_bits(fixed as $bits)
            }

            pub const fn from_bits(bits: $bits) -> Self {
                Self(bits)
            }

            pub const fn from_integer(value: $bits) -> Self {
                Self(((value as $wide) << $radix) as $bits)
            }

            pub const fn integer(self) -> Self {
                Self(((self.0 as $wide) >> $radix) as $bits)
            }

            pub const fn fraction(self) -> Self {
                Self(((self.0 as $wide) & ((1 << $radix) - 1)) as $bits)
            }

            pub const fn radix(self) -> u32 {
                $radix
            }

            pub const fn add(self, rhs: Self) -> Self {
                Self(self.0 + rhs.0)
            }

            pub const fn sub(self, rhs: Self) -> Self {
                Self(self.0 - rhs.0)
            }

            pub const fn mul(self, rhs: Self) -> Self {
                Self(((self.0 as $wide * rhs.0 as $wide) >> $radix) as $bits)
            }

            pub const fn div(self, rhs: Self) -> Self {
                Self((((self.0 as $wide) << $radix) / rhs.0 as $wide) as $bits)
            }

            pub const fn floor(self) -> Self {
                Self(((self.0 as $wide) & !((1 << $radix) - 1)) as $bits)
            }

            pub const fn ceil(self) -> Self {
                Self(((self.0 as $wide) + ((1 << $radix) - 1) & !((1 << $radix) - 1)) as $bits)
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
                let v = self.0 as $wide;
                // Print integer part
                write!(f, "{}.", v >> $radix)?;
                // Print fractional part
                let mut frac = (v & ((1 << $radix) - 1)) as $wide;
                while frac > 0 {
                    frac = frac.wrapping_mul(10);
                    write!(f, "{}", frac >> $radix)?;
                    frac &= (1 << $radix) - 1;
                }
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

define_unsigned!(U64F48, u64, 48, u128);
define_unsigned!(U64F32, u64, 32, u128);

define_signed!(I64F48, i64, 48, i128);
define_signed!(I64F32, i64, 32, i128);

define_unsigned!(U32F24, u32, 24, u64);
impl_float_conversion!(U32F24, u32, 24);
define_unsigned!(U32F16, u32, 16, u64);
impl_float_conversion!(U32F16, u32, 16);
define_unsigned!(U32F8, u32, 8, u64);
impl_float_conversion!(U32F8, u32, 8);

define_signed!(I32F24, i32, 24, i64);
impl_float_conversion!(I32F24, i32, 24);
define_signed!(I32F16, i32, 16, i64);
impl_float_conversion!(I32F16, i32, 16);
define_signed!(I32F8, i32, 8, i64);
impl_float_conversion!(I32F8, i32, 8);

define_unsigned!(U16F8, u16, 8, u32);
impl_float_conversion!(U16F8, u16, 8);
define_signed!(I16F8, i16, 8, i32);
impl_float_conversion!(I16F8, i16, 8);
define_unsigned!(U16F4, u16, 4, u32);
impl_float_conversion!(U16F4, u16, 4);
define_signed!(I16F4, i16, 4, i32);
impl_float_conversion!(I16F4, i16, 4);

define_unsigned!(U16F16, u16, 16, u32);

#[cfg(test)]
mod test {
    use std::println;

    use super::*;

    #[test]
    fn test_fixed() {
        let x = I32F24::from_f32(1.123123);
        println!("{}", x);
        println!("{}", I32F24::EPSILON);
        // let x2 = x.convert::<u32, 32>();
        // let x2 = x.convert::<u32, 8>();
        // let x3 = x2.convert::<u32, 16>();
        // println!("{}", x);
    }
}
