pub struct Color(u32);

impl Color {
    
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self((a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32))
    }

    pub fn r(&self) -> u8 {
        (self.0 & 0x00FF0000 >> 16) as u8
    }

    pub fn g(&self) -> u8 {
        (self.0 & 0x0000FF00 >> 8) as u8
    }

    pub fn b(&self) -> u8 {
        (self.0 & 0x000000FF) as u8
    }

    pub fn a(&self) -> u8 {
        (self.0 & 0xFF000000 >> 24) as u8
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