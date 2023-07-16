use mini3d_derive::{Error, Serialize};

use crate::{
    math::rect::IRect,
    renderer::{
        color::Color,
        graphics::{Graphics, TextureWrapMode},
    },
    utils::uid::UID,
};

#[derive(Debug, Error)]
pub enum UIImageStyleError {
    #[error("Invalid margin")]
    InvalidMargin,
}

#[derive(Serialize, Clone, Copy)]
pub struct UIMargin {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

impl UIMargin {
    pub fn new(top: u32, bottom: u32, left: u32, right: u32) -> Self {
        Self {
            top,
            bottom,
            left,
            right,
        }
    }

    pub fn zero() -> Self {
        Self {
            top: 0,
            bottom: 0,
            left: 0,
            right: 0,
        }
    }
}

#[derive(Serialize, Clone, Copy)]
pub struct UIImageStyle {
    texture: UID,
    extent: IRect,
    margin: UIMargin,
    wrap_mode: TextureWrapMode,
    center: bool,
}

impl UIImageStyle {
    pub(crate) fn render(
        &self,
        gfx: &mut Graphics,
        extent: IRect,
        filtering: Color,
        alpha_threshold: u8,
    ) {
        let tex_mid_width = self.extent.width() - self.margin.left - self.margin.right;
        let tex_mid_height = self.extent.height() - self.margin.top - self.margin.bottom;
        let ext_mid_width = extent.width() - self.margin.left - self.margin.right;
        let ext_mid_height = extent.height() - self.margin.top - self.margin.bottom;

        // Top-Left
        if self.margin.top > 0 && self.margin.left > 0 {
            let extent = IRect::new(
                extent.left(),
                extent.top(),
                self.margin.left,
                self.margin.top,
            );
            let texture_extent = IRect::new(
                self.extent.left(),
                self.extent.top(),
                self.margin.left,
                self.margin.top,
            );
            gfx.blit_texture(
                self.texture,
                extent,
                texture_extent,
                filtering,
                self.wrap_mode,
                alpha_threshold,
            );
        }

        // Top-Mid
        if self.margin.top > 0 {
            let extent = IRect::new(
                extent.left() + self.margin.left as i32,
                extent.top(),
                ext_mid_width,
                self.margin.top,
            );
            let texture_extent = IRect::new(
                self.extent.left() + self.margin.left as i32,
                self.extent.top(),
                tex_mid_height,
                self.margin.top,
            );
            gfx.blit_texture(
                self.texture,
                extent,
                texture_extent,
                filtering,
                self.wrap_mode,
                alpha_threshold,
            );
        }

        // Top-Right
        if self.margin.top > 0 && self.margin.right > 0 {
            let extent = IRect::new(
                extent.right() + 1 - self.margin.right as i32,
                extent.top(),
                self.margin.right,
                self.margin.top,
            );
            let texture_extent = IRect::new(
                self.extent.right() + 1 - self.margin.right as i32,
                self.extent.top(),
                self.margin.right,
                self.margin.top,
            );
            gfx.blit_texture(
                self.texture,
                extent,
                texture_extent,
                filtering,
                self.wrap_mode,
                alpha_threshold,
            );
        }

        // Mid-Left
        if self.margin.left > 0 {
            let extent = IRect::new(
                extent.left(),
                extent.top() + self.margin.top as i32,
                self.margin.left,
                ext_mid_height,
            );
            let texture_extent = IRect::new(
                self.extent.left(),
                self.extent.top() + self.margin.top as i32,
                self.margin.left,
                tex_mid_height,
            );
            gfx.blit_texture(
                self.texture,
                extent,
                texture_extent,
                filtering,
                self.wrap_mode,
                alpha_threshold,
            );
        }

        // Mid-Mid
        if self.center {
            let extent = IRect::new(
                extent.left() + self.margin.left as i32,
                extent.top() + self.margin.top as i32,
                ext_mid_width,
                ext_mid_height,
            );
            let texture_extent = IRect::new(
                self.extent.left() + self.margin.left as i32,
                self.extent.top() + self.margin.top as i32,
                tex_mid_width,
                tex_mid_height,
            );
            gfx.blit_texture(
                self.texture,
                extent,
                texture_extent,
                filtering,
                self.wrap_mode,
                alpha_threshold,
            );
        }

        // Mid-Right
        if self.margin.right > 0 {
            let extent = IRect::new(
                extent.right() + 1 - self.margin.right as i32,
                extent.top() + self.margin.top as i32,
                self.margin.right,
                ext_mid_height,
            );
            let texture_extent = IRect::new(
                self.extent.right() + 1 - self.margin.right as i32,
                self.extent.top() + self.margin.top as i32,
                self.margin.right,
                tex_mid_height,
            );
            gfx.blit_texture(
                self.texture,
                extent,
                texture_extent,
                filtering,
                self.wrap_mode,
                alpha_threshold,
            );
        }

        // Bottom-Left
        if self.margin.bottom > 0 && self.margin.left > 0 {
            let extent = IRect::new(
                extent.left(),
                extent.bottom() + 1 - self.margin.bottom as i32,
                self.margin.left,
                self.margin.bottom,
            );
            let texture_extent = IRect::new(
                self.extent.left(),
                self.extent.bottom() + 1 - self.margin.bottom as i32,
                self.margin.left,
                self.margin.bottom,
            );
            gfx.blit_texture(
                self.texture,
                extent,
                texture_extent,
                filtering,
                self.wrap_mode,
                alpha_threshold,
            );
        }

        // Bottom-Mid
        if self.margin.bottom > 0 {
            let extent = IRect::new(
                extent.left() + self.margin.left as i32,
                extent.bottom() + 1 - self.margin.bottom as i32,
                ext_mid_width,
                self.margin.bottom,
            );
            let texture_extent = IRect::new(
                self.extent.left() + self.margin.left as i32,
                self.extent.bottom() + 1 - self.margin.bottom as i32,
                tex_mid_width,
                self.margin.bottom,
            );
            gfx.blit_texture(
                self.texture,
                extent,
                texture_extent,
                filtering,
                self.wrap_mode,
                alpha_threshold,
            );
        }

        // Bottom-Right
        if self.margin.bottom > 0 && self.margin.right > 0 {
            let extent = IRect::new(
                extent.right() + 1 - self.margin.right as i32,
                extent.bottom() + 1 - self.margin.bottom as i32,
                self.margin.right,
                self.margin.bottom,
            );
            let texture_extent = IRect::new(
                self.extent.right() + 1 - self.margin.right as i32,
                self.extent.bottom() + 1 - self.margin.bottom as i32,
                self.margin.right,
                self.margin.bottom,
            );
            gfx.blit_texture(
                self.texture,
                extent,
                texture_extent,
                filtering,
                self.wrap_mode,
                alpha_threshold,
            );
        }
    }

    pub fn simple(texture: UID, extent: IRect) -> Self {
        Self {
            texture,
            extent,
            margin: UIMargin::zero(),
            wrap_mode: TextureWrapMode::Clamp,
            center: true,
        }
    }

    pub fn sliced(
        texture: UID,
        extent: IRect,
        margin: UIMargin,
    ) -> Result<Self, UIImageStyleError> {
        if margin.left + margin.right > extent.width()
            || margin.top + margin.bottom > extent.height()
        {
            return Err(UIImageStyleError::InvalidMargin);
        }
        Ok(Self {
            texture,
            extent,
            margin,
            wrap_mode: TextureWrapMode::Repeat,
            center: true,
        })
    }
}

#[derive(Serialize, Clone, Copy)]
pub enum UIBoxStyle {
    Color(Color),
    Image(UIImageStyle),
}

impl UIBoxStyle {
    pub(crate) fn render(
        &self,
        gfx: &mut Graphics,
        extent: IRect,
        filtering: Color,
        alpha_threshold: u8,
    ) {
        match self {
            UIBoxStyle::Color(color) => {
                gfx.fill_rect(extent, *color);
            }
            UIBoxStyle::Image(image) => {
                image.render(gfx, extent, filtering, alpha_threshold);
            }
        }
    }
}
