use alloc::vec::Vec;
use mini3d_derive::Serialize;
use mini3d_math::{
    rect::IRect,
    vec::{V2, V2U32},
};
use mini3d_utils::slot_map_key;

use crate::texture::TextureData;

use super::texture::TextureFormat;

slot_map_key!(FontId);

#[derive(Clone, Serialize)]
pub struct Font {
    pub glyph_size: V2U32,
    pub bytes: Vec<u8>,
    pub char_to_location: Vec<usize>,
}

impl Default for Font {
    fn default() -> Self {
        let glyph_width = 8;
        let glyph_height = 8;
        let data = include_bytes!("../../assets/font.bin").to_vec();
        let mut char_to_location = alloc::vec![0; Self::MAX_CHARS]; // Fill with default location
        for (i, c) in Self::CHARS.chars().enumerate() {
            char_to_location[c as usize] = i;
        }
        Self {
            glyph_size: V2::new(glyph_width as u32, glyph_height as u32),
            bytes: data,
            char_to_location,
        }
    }
}

impl Font {
    pub const CHARS: &'static str = " !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~éèê";
    pub const MAX_CHARS: usize = 256;

    pub fn char_location(&self, c: char) -> Option<usize> {
        if c as usize >= Self::MAX_CHARS {
            return None;
        }
        Some(self.char_to_location[c as usize])
    }
}

#[derive(Default)]
pub struct FontAtlas {
    pub texture: TextureData,
    pub extents: Vec<IRect>,
}

impl FontAtlas {
    pub fn new(font: &Font) -> FontAtlas {
        let width = font.glyph_size.x as usize * Font::MAX_CHARS;
        let height = font.glyph_size.y as usize;
        let mut texture = TextureData {
            bytes: alloc::vec![0x0; width * height * 4],
            format: TextureFormat::Color,
            width: width as u16,
            height: height as u16,
            usage: Default::default(),
        };

        let mut extents = alloc::vec![IRect::default(); Font::MAX_CHARS];
        let mut extent = IRect::new(0, 0, font.glyph_size.x, height as u32);
        for (c, location) in Font::CHARS
            .chars()
            .map(|c| (c, font.char_to_location[c as usize]))
        {
            // Write pixels to texture
            // TODO: optimize me
            for p in 0..(font.glyph_size.x as usize * font.glyph_size.y as usize) {
                let bit_offset =
                    (location * (font.glyph_size.x as usize * font.glyph_size.y as usize)) + p;
                let byte = font.bytes[bit_offset / 8];
                let bit_set = byte & (1 << (7 - (p % 8))) != 0;

                let px = (extent.left() + (p as i32 % font.glyph_size.x as i32)) as usize;
                let py = (extent.top() + (p as i32 / font.glyph_size.x as i32)) as usize;
                let pi = py * texture.width as usize + px;
                let color = if bit_set { 0xFF } else { 0x0 };
                texture.bytes[pi * 4] = color;
                texture.bytes[pi * 4 + 1] = color;
                texture.bytes[pi * 4 + 2] = color;
                texture.bytes[pi * 4 + 3] = color;
            }

            // Save extent and move to next glyph
            extents[c as usize] = extent;
            extent = extent.translate(V2::new(font.glyph_size.x as i32, 0));
        }
        Self { texture, extents }
    }
}
