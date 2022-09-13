use glam::{UVec2, uvec2, IVec2, Mat4};

use crate::{math::rect::IRect, asset::AssetId};

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

pub type ModelId = u32;

pub struct Model {
    id: ModelId,
    transform: Mat4,
    mesh: AssetId,
    materials: Vec<AssetId>,
}

#[derive(Default)]
pub struct Graphics {
    pub(crate) immediate_commands: Vec<ImmediateCommand>,
    pub(crate) models: Vec<Model>,
    next_id: u32,
}

impl Graphics {
    pub(crate) fn print(&mut self, p: IVec2, text: &str, font_id: AssetId) {
        self.immediate_commands.push(ImmediateCommand::Print { p, text: String::from(text), font_id })
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
        self.immediate_commands.push(ImmediateCommand::DrawRect { rect });
    }
    pub(crate) fn fill_rect(&mut self, rect: IRect) {
        self.immediate_commands.push(ImmediateCommand::FillRect { rect });
    }

    pub(crate) fn add_model(&mut self) -> ModelId {
        0
    }
    pub(crate) fn remove_model(&mut self, id: ModelId) {
        
    }

    pub fn immediate_commands(&self) -> &Vec<ImmediateCommand> {
        &self.immediate_commands
    }
    pub fn models(&self) -> &Vec<Model> {
        &self.models
    }
}