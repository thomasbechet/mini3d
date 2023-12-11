use core::fmt::Debug;

use mini3d_derive::{fixed, Serialize};

use crate::math::fixed::I32F16;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Color(u32);

impl Debug for Color {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let r = self.r();
        let g = self.g();
        let b = self.b();
        let a = self.a();
        f.debug_tuple("Color")
            .field(&r)
            .field(&g)
            .field(&b)
            .field(&a)
            .finish()
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

impl Color {
    pub const WHITE: Color = Color::rgba(255, 255, 255, 255);
    pub const BLACK: Color = Color::rgba(0, 0, 0, 255);
    pub const RED: Color = Color::rgba(255, 0, 0, 255);
    pub const GREEN: Color = Color::rgba(0, 255, 0, 255);
    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
    pub const GRAY: Color = Color::rgba(200, 200, 200, 255);

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self((a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32))
    }

    pub fn r(&self) -> u8 {
        ((self.0 & 0x00FF0000) >> 16) as u8
    }

    pub fn g(&self) -> u8 {
        ((self.0 & 0x0000FF00) >> 8) as u8
    }

    pub fn b(&self) -> u8 {
        (self.0 & 0x000000FF) as u8
    }

    pub fn a(&self) -> u8 {
        ((self.0 & 0xFF000000) >> 24) as u8
    }
}

impl From<Color> for [I32F16; 4] {
    fn from(color: Color) -> Self {
        let r = I32F16::from(color.r()) / fixed!(255i32f16);
        let g = I32F16::from(color.g()) / fixed!(255i32f16);
        let b = I32F16::from(color.b()) / fixed!(255i32f16);
        let a = I32F16::from(color.a()) / fixed!(255i32f16);
        [r, g, b, a]
    }
}

impl From<Color> for [I32F16; 3] {
    fn from(color: Color) -> Self {
        let r = I32F16::from(color.r()) / fixed!(255i32f16);
        let g = I32F16::from(color.g()) / fixed!(255i32f16);
        let b = I32F16::from(color.b()) / fixed!(255i32f16);
        [r, g, b]
    }
}

pub fn linear_to_srgb(c: [I32F16; 3]) -> [I32F16; 3] {
    let f = |x: I32F16| -> I32F16 {
        if x > fixed!(0.0031308) {
            let a = fixed!(0.055i32f16);
            (fixed!(1i32f16) + a) * x.pow(fixed!(-2.4)) - a
        } else {
            fixed!(12.92i32f16) * x
        }
    };
    [f(c[0]), f(c[1]), f(c[2])]
}

pub fn srgb_to_linear(c: [I32F16; 3]) -> [I32F16; 3] {
    let f = |x: I32F16| -> I32F16 {
        if x > fixed!(0.04045) {
            ((x + fixed!(0.055i32f16)) / fixed!(1.055i32f16)).pow(fixed!(2.4))
        } else {
            x / fixed!(12.92i32f16)
        }
    };
    [f(c[0]), f(c[1]), f(c[2])]
}
