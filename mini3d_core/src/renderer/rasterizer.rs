use crate::math::{
    rect::IRect,
    vec::{V2, V2I32},
};

use super::component::Font;

pub trait Plotable {
    /// Request a single point plot from the rasterizer
    ///
    /// # Arguments
    ///
    /// * `p` - Point coordinates
    fn plot(&mut self, p: V2I32);
}

pub fn draw_line(mut p0: V2I32, p1: V2I32, mut plot: impl FnMut(V2I32)) {
    let dx = p0.x.abs_diff(p1.x) as i32;
    let sx: i32 = if p0.x < p1.x { 1 } else { -1 };
    let dy = -(p0.y.abs_diff(p1.y) as i32);
    let sy: i32 = if p0.y < p1.y { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        plot(p0);
        if p0.x == p1.x && p0.y == p1.y {
            break;
        }
        let e2 = 2 * error;
        if e2 >= dy {
            if p0.x == p1.x {
                break;
            }
            error += dy;
            p0.x += sx;
        }
        if e2 <= dx {
            if p0.y == p1.y {
                break;
            }
            error += dx;
            p0.y += sy;
        }
    }
}

pub fn draw_vline(p: &mut impl Plotable, x: i32, y0: i32, y1: i32) {
    for y in y0..=y1 {
        p.plot(V2::new(x, y));
    }
}

pub fn draw_hline(p: &mut impl Plotable, y: i32, x0: i32, x1: i32) {
    for x in x0..=x1 {
        p.plot(V2::new(x, y));
    }
}

pub fn print(plot: &mut impl Plotable, p: V2I32, text: &str, font: &Font) {
    for (ic, c) in text.chars().enumerate() {
        if let Some(glyph) = font.char_location(c) {
            // TODO: optimize me
            for b in 0..(font.data.glyph_size.x as usize * font.data.glyph_size.y as usize) {
                let bit_offset = (glyph
                    * (font.data.glyph_size.x as usize * font.data.glyph_size.y as usize))
                    + b;
                let byte = font.data.bytes[bit_offset / 8];
                let bit_set = byte & (1 << (7 - (b % 8))) != 0;

                if bit_set {
                    let px = p.x
                        + font.data.glyph_size.x as i32 * ic as i32
                        + (b as i32 % font.data.glyph_size.x as i32);
                    let py = p.y + (b as i32 / font.data.glyph_size.x as i32);
                    plot.plot(V2::new(px, py));
                }
            }
        }
    }
}

pub fn draw_rect(p: &mut impl Plotable, rect: IRect) {
    let p0 = rect.tl();
    let p1 = rect.br();
    draw_hline(p, p0.y, p0.x, p1.x);
    draw_hline(p, p1.y, p0.x, p1.x);
    draw_vline(p, p0.x, p0.y + 1, p1.y - 1);
    draw_vline(p, p1.x, p0.y + 1, p1.y - 1);
}

pub fn fill_rect(p: &mut impl Plotable, rect: IRect) {
    let p0 = rect.tl();
    let p1 = rect.br();
    for y in p0.y..=p1.y {
        for x in p0.x..=p1.x {
            p.plot(V2::new(x, y));
        }
    }
}
