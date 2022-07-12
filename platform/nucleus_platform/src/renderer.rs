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

#[derive(Debug, Copy, Clone)]
pub struct Point {
    x: u16,
    y: u16
}

#[derive(Debug)]
pub enum RendererError {
    Timeout,
    Outdated,
    Lost,
    OutOfMemory
}

pub trait Renderer: Sized {
    fn render(&mut self) -> Result<(), RendererError>;
    fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError>;
}

pub trait Graphics2D {
    fn fill_background(&mut self);
    fn draw_line(&mut self, src: Point, dst: Point);
    fn draw_rect(&mut self, a: Point, b: Point);
    fn draw_rect_tiled(&mut self, a: Point, b: Point);
    fn fill_rect(&mut self, a: Point, b: Point);
    fn fill_rect_tiled(&mut self, a: Point, b: Point);
    fn print_text(&mut self, p: Point, text: &str);
    fn print_text_tiled(&mut self, p: Point, text: &str);
    fn draw_viewport(&mut self);
}