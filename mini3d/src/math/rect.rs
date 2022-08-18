use glam::IVec2;

/// Basic rectangle structure with useful functions
/// Vec4: xy -> top-left, zw -> bottom-right
#[derive(Clone, Copy)]
pub struct IRect {
    tl: IVec2,
    br: IVec2,
}

impl IRect {

    pub const fn new(x: i32, y: i32, width: u32, height: u32) -> Self {
        IRect {
            tl: IVec2::new(x, y),
            br: IVec2::new(x - 1 + width as i32, y - 1 + height as i32)
        }
    }

    /// Top-Left point
    #[inline]
    pub fn tl(&self) -> IVec2 {
        self.tl
    }

    /// Bottom-Right point
    #[inline]
    pub fn br(&self) -> IVec2 {
        self.br
    }

    /// Top-Right point
    #[inline]
    pub fn tr(&self) -> IVec2 {
        IVec2::new(self.br.x, self.tl.y)
    }

    /// Bottom-Left point
    #[inline]
    pub fn bl(&self) -> IVec2 {
        IVec2::new(self.tl.x, self.br.y)
    }

    /// Center-Up point
    #[inline]
    pub fn cu(&self) -> IVec2 {
        IVec2::new((self.tl.x + self.br.x) / 2, self.tl.y)
    }

    /// Center-Bottom point
    #[inline]
    pub fn cb(&self) -> IVec2 {
        IVec2::new((self.tl.x + self.br.x) / 2, self.br.y)
    }

    /// Center-Right point
    #[inline]
    pub fn cr(&self) -> IVec2 {
        IVec2::new(self.br.x, (self.tl.y + self.br.y) / 2)
    }

    /// Center-Right point
    #[inline]
    pub fn cl(&self) -> IVec2 {
        IVec2::new(self.tl.x, (self.tl.y + self.br.y) / 2)
    }

    #[inline]
    pub fn center(&self) -> IVec2 {
        IVec2::new((self.tl.x + self.br.x) / 2, (self.tl.y + self.br.y) / 2)
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.br.x as u32 - self.tl.x as u32
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.br.y as u32 - self.tl.y as u32
    }

    #[inline]
    pub fn contains(&self, p: IVec2) -> bool {
        p.x >= self.tl.x && p.y >= self.tl.y && p.x <= self.br.x && p.y <= self.br.y
    }

    #[inline]
    pub fn clamp(&mut self, rect: &IRect) {
        self.tl = self.tl.max(rect.tl);
        self.br = self.br.min(rect.br);
    }
}