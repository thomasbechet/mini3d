use std::collections::HashMap;
use bitvec::prelude::*;
use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{math::rect::IRect, registry::asset::Asset, uid::UID};

use super::texture::{Texture, TextureFormat};

#[derive(Clone, Serialize, Deserialize)]
pub struct Font {
    pub glyph_width: u8,
    pub glyph_height: u8,
    pub data: BitVec<u8, Msb0>,
    pub glyph_locations: HashMap<char, usize>,
}

impl Default for Font {
    fn default() -> Self {
        let glyph_width = 8;
        let glyph_height = 8;
        let data = BitVec::from_slice(include_bytes!("../../../../assets/font.bin"));
        let glyph_locations: HashMap<_, _> = " !\"#$%&\'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[]^_`abcdefghijklmnopqrstuvwxyz{|}~éèê"
            .chars().enumerate().map(|(i, x)| (x, i * (glyph_height * glyph_width))).collect();
        Font {
            glyph_width: glyph_width as u8,
            glyph_height: glyph_height as u8,
            data,
            glyph_locations,
        }
    }
}

impl Asset for Font {}

impl Font {
    pub const NAME: &'static str = "font";
    pub const UID: UID = UID::new(Font::NAME);
}

pub struct FontAtlas {
    pub texture: Texture,
    pub extents: HashMap<char, IRect>,
}

impl FontAtlas {
    pub fn new(font: &Font) -> FontAtlas {
        let glyph_count = font.glyph_locations.len();
        let width = font.glyph_width as u32 * glyph_count as u32;
        let height = font.glyph_height as u32;
        let mut texture = Texture {
            data: vec![0x0; (width * height * 4) as usize],
            format: TextureFormat::RGBA,
            width,
            height,
        };

        let mut extents: HashMap<char, IRect> = Default::default();
        let mut extent = IRect::new(0, 0, font.glyph_width as u32, height);
        for (c, location) in &font.glyph_locations {

            // Write glyph pixels
            let start = *location;
            let end = start + (font.glyph_width as usize * font.glyph_height as usize);
            for (i, b) in font.data.as_bitslice()[start..end].iter().enumerate() {
                let px = (extent.left() + (i as i32 % font.glyph_width as i32)) as usize;
                let py = (extent.top() + (i as i32 / font.glyph_width as i32)) as usize;
                let pi = py * texture.width as usize + px;
                let byte = if *b { 0xFF } else { 0x0 };
                texture.data[pi * 4] = byte;
                texture.data[pi * 4 + 1] = byte;
                texture.data[pi * 4 + 2] = byte;
                texture.data[pi * 4 + 3] = byte;
            }

            // Save extent and move to next glyph
            extents.insert(*c, extent);
            extent.translate(IVec2::new(font.glyph_width as i32, 0));
        }

        Self { texture, extents }
    }
}