use mini3d_derive::Serialize;

use super::vec::{V2, V2I32};

/// Basic rectangle structure with useful functions
/// Vec4: xy -> top-left, zw -> bottom-right
#[derive(Default, Debug, Clone, Copy, Serialize)]
pub struct IRect {
    tl: V2I32,
    br: V2I32,
}

impl IRect {
    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        IRect {
            tl: V2::new(x, y),
            br: V2::new(x - 1 + width as i32, y - 1 + height as i32),
        }
    }

    /// Top-Left point
    #[inline]
    pub fn tl(&self) -> V2I32 {
        self.tl
    }

    /// Bottom-Right point
    #[inline]
    pub fn br(&self) -> V2I32 {
        self.br
    }

    /// Top-Right point
    #[inline]
    pub fn tr(&self) -> V2I32 {
        V2::new(self.br.x, self.tl.y)
    }

    /// Bottom-Left point
    #[inline]
    pub fn bl(&self) -> V2I32 {
        V2::new(self.tl.x, self.br.y)
    }

    /// Center-Up point
    #[inline]
    pub fn cu(&self) -> V2I32 {
        V2::new((self.tl.x + self.br.x) / 2, self.tl.y)
    }

    /// Center-Bottom point
    #[inline]
    pub fn cb(&self) -> V2I32 {
        V2::new((self.tl.x + self.br.x) / 2, self.br.y)
    }

    /// Center-Right point
    #[inline]
    pub fn cr(&self) -> V2I32 {
        V2::new(self.br.x, (self.tl.y + self.br.y) / 2)
    }

    /// Center-Right point
    #[inline]
    pub fn cl(&self) -> V2I32 {
        V2::new(self.tl.x, (self.tl.y + self.br.y) / 2)
    }

    #[inline]
    pub fn center(&self) -> V2I32 {
        V2::new((self.tl.x + self.br.x) / 2, (self.tl.y + self.br.y) / 2)
    }

    #[inline]
    pub fn top(&self) -> i32 {
        self.tl.y
    }

    #[inline]
    pub fn bottom(&self) -> i32 {
        self.br.y
    }

    #[inline]
    pub fn left(&self) -> i32 {
        self.tl.x
    }

    #[inline]
    pub fn right(&self) -> i32 {
        self.br.x
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.br.x as u32 - self.tl.x as u32 + 1
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.br.y as u32 - self.tl.y as u32 + 1
    }

    #[inline]
    pub fn size(&self) -> V2U32 {
        V2::new(self.width(), self.height())
    }

    #[inline]
    pub fn area(&self) -> u32 {
        self.width() * self.height()
    }

    #[inline]
    pub fn contains(&self, p: V2I32) -> bool {
        p.x >= self.tl.x && p.y >= self.tl.y && p.x <= self.br.x && p.y <= self.br.y
    }

    #[inline]
    pub fn clamp(self, rect: IRect) -> Self {
        Self {
            tl: self.tl.max(rect.tl),
            br: self.br.min(rect.br),
        }
    }

    #[inline]
    pub fn lerp(self, rect: IRect, a: f32) -> Self {
        Self {
            tl: self.tl.as_vec2().lerp(rect.tl.as_vec2(), a).as_ivec2(),
            br: self.br.as_vec2().lerp(rect.br.as_vec2(), a).as_ivec2(),
        }
    }

    #[inline]
    pub fn translate(self, translation: V2I32) -> Self {
        Self {
            tl: self.tl + translation,
            br: self.br + translation,
        }
    }
}

impl From<(i32, i32, u32, u32)> for IRect {
    fn from(rect: (i32, i32, u32, u32)) -> Self {
        Self::new(rect.0, rect.1, rect.2, rect.3)
    }
}
