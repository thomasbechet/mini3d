use std::collections::HashMap;
use bitvec::array::BitArray;

type Glyph = BitArray;

pub struct Font {
    pub glyph_width: u16,
    pub glyph_height: u16,
    pub glyphs: HashMap<char, Glyph>,
}