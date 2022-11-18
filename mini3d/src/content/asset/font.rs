use std::collections::HashMap;
use bitvec::prelude::*;
use serde::{Serialize, Deserialize};

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