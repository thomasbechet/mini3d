use crate::asset::font::Font;

pub trait Plotable {
    fn plot(&mut self, x: u16, y: u16);
}

pub fn draw_line(p: &mut impl Plotable, mut x0: u16, mut y0: u16, x1: u16, y1: u16) {
    let dx = x0.abs_diff(x1) as i16;
    let sx: i16 = if x0 < x1 { 1 } else { -1 };
    let dy = -(y0.abs_diff(y1) as i16);
    let sy: i16 = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        p.plot(x0, y0);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * error;
        if e2 >= dy {
            if x0 == x1 {
                break;
            }
            error += dy;
            x0 += sx as u16;
        }
        if e2 <= dx {
            if y0 == y1 {
                break;
            }
            error += dx;
            y0 = (y0 as i16 + sy) as u16;
        }
    }
}

pub fn draw_vline(p: &mut impl Plotable, x: u16, y0: u16, y1: u16) {
    for y in y0..=y1 {
        p.plot(x, y);
    }
}

pub fn draw_hline(p: &mut impl Plotable, y: u16, x0: u16, x1: u16) {
    for x in x0..=x1 {
        p.plot(x, y);
    }
}

pub fn print(p: &mut impl Plotable, x: u16, y: u16, text: &str, font: &Font) {
    for (ic, c) in text.chars().enumerate() {
        if let Some(glyph) = font.glyph_locations.get(&c) {
            let start = *glyph;
            let end = start + (font.glyph_width as usize * font.glyph_height as usize);
            for (i, b) in font.data.as_bitslice()[start..end].iter().enumerate() {
                if *b {
                    let px = x + font.glyph_width as u16 * ic as u16 + (i as u16 % font.glyph_width as u16);
                    let py = y + (i as u16 / font.glyph_width as u16);
                    p.plot(px, py);
                }
            }
        }
    }
}

pub fn draw_rect(p: &mut impl Plotable, x0: u16, y0: u16, x1: u16, y1: u16) {
    draw_hline(p, y0, x0, x1);
    draw_hline(p, y1, x0, x1);
    draw_vline(p, x0, y0 + 1, y1 - 1);
    draw_vline(p, x1, y0 + 1, y1 - 1);
}

pub fn fill_rect(p: &mut impl Plotable, x0: u16, y0: u16, x1: u16, y1: u16) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            p.plot(x, y);
        }
    }
}