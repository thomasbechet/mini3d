use glam::{UVec2, uvec2, IVec2};

use crate::{asset::{font::Font, Asset}, math::rect::IRect};

// 3:2 aspect ratio
// pub const SCREEN_WIDTH: u32 = 480;
// pub const SCREEN_HEIGHT: u32 = 320;
// // 4:3 aspect ratio
// pub const SCREEN_WIDTH: u32 = 512;
// pub const SCREEN_HEIGHT: u32 = 384;
// // 16:10 aspect ratio
pub const SCREEN_WIDTH: u32 = 432;
pub const SCREEN_HEIGHT: u32 = 240;
// // 16:9 aspect ratio
// pub const SCREEN_WIDTH: u32 = 384;
// pub const SCREEN_HEIGHT: u32 = 216;

pub const SCREEN_PIXEL_COUNT: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize;
pub const SCREEN_RESOLUTION: UVec2 = uvec2(SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_VIEWPORT: IRect = IRect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);

#[derive(Debug)]
pub enum RendererError {
    Timeout,
    Outdated,
    Lost,
    OutOfMemory
}

pub trait RendererService {
    fn present(&mut self) -> Result<(), RendererError>;
    fn resize(&mut self, width: u32, height: u32);
    fn print(&mut self, p: IVec2, text: &str, font: &Asset<Font>);
    fn draw_line(&mut self, p0: IVec2, p1: IVec2);
    fn draw_vline(&mut self, x: i32, y0: i32, y1: i32);
    fn draw_hline(&mut self, y: i32, x0: i32, x1: i32);
    fn draw_rect(&mut self, rect: IRect);
    fn fill_rect(&mut self, rect: IRect);
    fn clear(&mut self);
}