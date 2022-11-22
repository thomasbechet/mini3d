use glam::{UVec2, IVec2};

use crate::{feature::asset::font::Font, math::rect::IRect};

pub trait Plotable {
    /// Request a single point plot from the rasterizer
    /// 
    /// # Arguments
    /// 
    /// * `p` - Point coordinates (assumed in the screen resolution)
    fn plot(&mut self, p: UVec2);
}

pub fn draw_line(p: &mut impl Plotable, mut p0: IVec2, p1: IVec2) {
    let dx = p0.x.abs_diff(p1.x) as i32;
    let sx: i32 = if p0.x < p1.x { 1 } else { -1 };
    let dy = -(p0.y.abs_diff(p1.y) as i32);
    let sy: i32 = if p0.y < p1.y { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        p.plot(p0.as_uvec2());
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
        p.plot(UVec2::new(x as u32, y as u32));
    }
}

pub fn draw_hline(p: &mut impl Plotable, y: i32, x0: i32, x1: i32) {
    for x in x0..=x1 {
        p.plot(UVec2::new(x as u32, y as u32));
    }
}

pub fn print(plot: &mut impl Plotable, p: IVec2, text: &str, font: &Font) {
    for (ic, c) in text.chars().enumerate() {
        if let Some(glyph) = font.glyph_locations.get(&c) {
            let start = *glyph;
            let end = start + (font.glyph_width as usize * font.glyph_height as usize);
            for (i, b) in font.data.as_bitslice()[start..end].iter().enumerate() {
                if *b {
                    let px = p.x as u32 + font.glyph_width as u32 * ic as u32 + (i as u32 % font.glyph_width as u32);
                    let py = p.y as u32 + (i as u32 / font.glyph_width as u32);
                    plot.plot(UVec2::new(px, py));
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
            p.plot(UVec2::new(x as u32, y as u32));
        }
    }
}