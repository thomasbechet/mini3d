use crate::asset::{font::Font, Asset};

// 3:2 aspect ration
pub const DISPLAY_WIDTH: u16 = 480;
pub const DISPLAY_HEIGHT: u16 = 320;
// // 4:3 aspect ration
// pub const DISPLAY_WIDTH: u16 = 512;
// pub const DISPLAY_HEIGHT: u16 = 384;
// // 16:10 aspect ration
// pub const DISPLAY_WIDTH: u16 = 432;
// pub const DISPLAY_HEIGHT: u16 = 240;

pub const DISPLAY_PIXEL_COUNT: usize = DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize;

#[derive(Debug)]
pub enum RendererError {
    Timeout,
    Outdated,
    Lost,
    OutOfMemory
}

pub trait RendererService {
    fn render(&mut self) -> Result<(), RendererError>;
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError>;
    fn print(&mut self, x: u16, y: u16, text: &str, font: &Asset<Font>) -> Result<(), RendererError>;
    fn draw_line(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), RendererError>;
    fn draw_vline(&mut self, x: u16, y0: u16, y1: u16) -> Result<(), RendererError>;
    fn draw_hline(&mut self, y: u16, x0: u16, x1: u16) -> Result<(), RendererError>;
    fn draw_rect(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), RendererError>;
    fn fill_rect(&mut self, x0: u16, y0: u16, x1: u16, y1: u16) -> Result<(), RendererError>;
    fn clear(&mut self) -> Result<(), RendererError>;
}