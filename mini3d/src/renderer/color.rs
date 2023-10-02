use std::fmt::Debug;

use mini3d_derive::Serialize;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub struct Color(u32);

impl Debug for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        let r = color.r() as f32 / 255.0;
        let g = color.g() as f32 / 255.0;
        let b = color.b() as f32 / 255.0;
        let a = color.a() as f32 / 255.0;
        [r, g, b, a]
    }
}

impl From<Color> for [f64; 4] {
    fn from(color: Color) -> Self {
        let r = color.r() as f64 / 255.0;
        let g = color.g() as f64 / 255.0;
        let b = color.b() as f64 / 255.0;
        let a = color.a() as f64 / 255.0;
        [r, g, b, a]
    }
}

impl From<Color> for [f32; 3] {
    fn from(color: Color) -> Self {
        let r = color.r() as f32 / 255.0;
        let g = color.g() as f32 / 255.0;
        let b = color.b() as f32 / 255.0;
        [r, g, b]
    }
}

pub fn linear_to_srgb(c: [f32; 3]) -> [f32; 3] {
    let f = |x: f32| -> f32 {
        if x > 0.0031308 {
            let a = 0.055;
            (1.0 + a) * x.powf(-2.4) - a
        } else {
            12.92 * x
        }
    };
    [f(c[0]), f(c[1]), f(c[2])]
}

pub fn srgb_to_linear(c: [f32; 3]) -> [f32; 3] {
    let f = |x: f32| -> f32 {
        if x > 0.04045 {
            ((x + 0.055) / 1.055).powf(2.4)
        } else {
            x / 12.92
        }
    };
    [f(c[0]), f(c[1]), f(c[2])]
}
