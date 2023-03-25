use serde::{Serialize, Deserialize};

use crate::{uid::UID, math::rect::IRect, renderer::{graphics::{TextureWrapMode, Graphics}, color::Color}};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct UIMargin {
    pub top: u32,
    pub bottom: u32,
    pub left: u32,
    pub right: u32,
}

impl UIMargin {
    pub fn new(top: u32, bottom: u32, left: u32, right: u32) -> Self {
        Self { top, bottom, left, right }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum UIBoxStyle {
    Simple {
        texture: UID,
        texture_extent: IRect,
        wrap_mode: TextureWrapMode,
    },
    Sliced {
        texture: UID,
        texture_extent: IRect,
        margin: UIMargin,
        wrap_mode: TextureWrapMode,
    },
    Tiled {

    },
}

impl UIBoxStyle {

    pub fn simple(texture: UID, texture_extent: IRect, wrap: TextureWrapMode) -> Self {
        Self::Simple { texture, texture_extent, wrap_mode: wrap }
    }

    pub fn sliced(texture: UID, texture_extent: IRect, margin: UIMargin, wrap: TextureWrapMode) -> Self {
        Self::Sliced { texture, texture_extent, margin, wrap_mode: wrap }
    }

    pub(crate) fn render(&self, gfx: &mut Graphics, extent: IRect, filtering: Color, alpha_threshold: u8) {
        match self {
            UIBoxStyle::Simple { texture, texture_extent, wrap_mode } => {
                gfx.blit_texture(*texture, extent, *texture_extent, filtering, *wrap_mode, alpha_threshold);
            },
            UIBoxStyle::Sliced { texture, texture_extent, margin, wrap_mode } => {
                
                let tex_mid_width = texture_extent.width() - margin.left - margin.right;
                let tex_mid_height = texture_extent.height() - margin.top - margin.bottom;
                let ext_mid_width = extent.width() - margin.left - margin.right;
                let ext_mid_height = extent.height() - margin.top - margin.bottom;

                // Top-Left
                {
                    let extent = IRect::new(extent.left(), extent.top(), margin.left, margin.top);
                    let texture_extent = IRect::new(texture_extent.left(), texture_extent.top(), margin.left, margin.top);
                    gfx.blit_texture(*texture, extent, texture_extent, filtering, *wrap_mode, alpha_threshold);
                }

                // Top-Mid
                {
                    let extent = IRect::new(extent.left() + margin.left as i32, extent.top(), ext_mid_width, margin.top);
                    let texture_extent = IRect::new(texture_extent.left() + margin.left as i32, texture_extent.top(), tex_mid_height, margin.top);
                    gfx.blit_texture(*texture, extent, texture_extent, filtering, *wrap_mode, alpha_threshold);
                }

                // Top-Right
                {
                    let extent = IRect::new(extent.right() - margin.right as i32, extent.top(), margin.right, margin.top);
                    let texture_extent = IRect::new(texture_extent.right() - margin.right as i32, texture_extent.top(), margin.right, margin.top);
                    gfx.blit_texture(*texture, extent, texture_extent, filtering, *wrap_mode, alpha_threshold);
                }

                // Mid-Left
                {
                    let extent = IRect::new(extent.left(), extent.top() + margin.top as i32, margin.left, ext_mid_height);
                    let texture_extent = IRect::new(texture_extent.left(), texture_extent.top() + margin.top as i32, margin.left, tex_mid_height);
                    gfx.blit_texture(*texture, extent, texture_extent, filtering, *wrap_mode, alpha_threshold);
                }

                // Mid-Mid
                {
                    let extent = IRect::new(extent.left() + margin.left as i32, extent.top() + margin.top as i32, ext_mid_width, ext_mid_height);
                    let texture_extent = IRect::new(texture_extent.left() + margin.left as i32, texture_extent.top() + margin.top as i32, tex_mid_width, tex_mid_height);
                    gfx.blit_texture(*texture, extent, texture_extent, filtering, *wrap_mode, alpha_threshold);
                }

                // Mid-Right
                {
                    let extent = IRect::new(extent.right() - margin.right as i32, extent.top() + margin.top as i32, margin.right, ext_mid_height);
                    let texture_extent = IRect::new(texture_extent.right() - margin.right as i32, texture_extent.top() + margin.top as i32, margin.right, tex_mid_height);
                    gfx.blit_texture(*texture, extent, texture_extent, filtering, *wrap_mode, alpha_threshold);
                }

                // Bottom-Left
                {
                    let extent = IRect::new(extent.left(), extent.bottom() - margin.bottom as i32, margin.left, margin.bottom);
                    let texture_extent = IRect::new(texture_extent.left(), texture_extent.bottom() - margin.bottom as i32, margin.left, margin.bottom);
                    gfx.blit_texture(*texture, extent, texture_extent, filtering, *wrap_mode, alpha_threshold);
                }

                // Bottom-Mid
                {
                    let extent = IRect::new(extent.left() + margin.left as i32, extent.bottom() - margin.bottom as i32, ext_mid_width, margin.bottom);
                    let texture_extent = IRect::new(texture_extent.left() + margin.left as i32, texture_extent.bottom() - margin.bottom as i32, tex_mid_width, margin.bottom);
                    gfx.blit_texture(*texture, extent, texture_extent, filtering, *wrap_mode, alpha_threshold);
                }

                // Bottom-Right
                {
                    let extent = IRect::new(extent.right() - margin.right as i32, extent.bottom() - margin.bottom as i32, margin.right, margin.bottom);
                    let texture_extent = IRect::new(texture_extent.right() - margin.right as i32, texture_extent.bottom() - margin.bottom as i32, margin.right, margin.bottom);
                    gfx.blit_texture(*texture, extent, texture_extent, filtering, *wrap_mode, alpha_threshold);
                }
            },
            UIBoxStyle::Tiled {  } => {

            },
        }
    }
}