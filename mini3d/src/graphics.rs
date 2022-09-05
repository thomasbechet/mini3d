use glam::{UVec2, uvec2, IVec2};

use crate::{math::rect::IRect, asset::AssetID};

use self::immediate_command::ImmediateCommand;

pub mod rasterizer;
pub mod immediate_command;

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
pub const SCREEN_ASPECT_RATIO: f32 = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;

#[derive(Default)]
pub struct Graphics {
    pub commands: Vec<ImmediateCommand>
}

impl Graphics {
    pub(crate) fn print(&mut self, p: IVec2, text: &str, font_id: AssetID) {
        self.commands.push(ImmediateCommand::Print { p, text: String::from(text), font_id })
    }
    // pub(crate) fn draw_line(&mut self, p0: IVec2, p1: IVec2) {
    //     self.commands.push(ImmediateCommand::DrawLine { p0, p1 });
    // }
    // pub(crate) fn draw_vline(&mut self, x: i32, y0: i32, y1: i32) {
    //     self.commands.push(ImmediateCommand::DrawVLine { x, y0, y1 });
    // }
    // pub(crate) fn draw_hline(&mut self, y: i32, x0: i32, x1: i32) {
    //     self.commands.push(ImmediateCommand::DrawHLine { y, x0, x1 });
    // }
    pub(crate) fn draw_rect(&mut self, rect: IRect) {
        self.commands.push(ImmediateCommand::DrawRect { rect });
    }
    pub(crate) fn fill_rect(&mut self, rect: IRect) {
        self.commands.push(ImmediateCommand::FillRect { rect });
    }
}