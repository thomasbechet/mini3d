use glam::{UVec2, uvec2, IVec2};
use slotmap::new_key_type;

use crate::{math::rect::IRect, asset::font::FontId};

use self::command_buffer::Command;

pub mod rasterizer;
pub mod command_buffer;

// 3:2 aspect ratio
// pub const SCREEN_WIDTH: u32 = 480;
// pub const SCREEN_HEIGHT: u32 = 320;
// // 4:3 aspect ratio
// pub const SCREEN_WIDTH: u32 = 512;
// pub const SCREEN_HEIGHT: u32 = 384;
// // 16:10 aspect ratio
// pub const SCREEN_WIDTH: u32 = 320;
// pub const SCREEN_HEIGHT: u32 = 200;
pub const SCREEN_WIDTH: u32 = 640;
pub const SCREEN_HEIGHT: u32 = 400;
// // 16:9 aspect ratio
// pub const SCREEN_WIDTH: u32 = 384;
// pub const SCREEN_HEIGHT: u32 = 216;

pub const SCREEN_PIXEL_COUNT: usize = SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize;
pub const SCREEN_RESOLUTION: UVec2 = uvec2(SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_VIEWPORT: IRect = IRect::new(0, 0, SCREEN_WIDTH, SCREEN_HEIGHT);
pub const SCREEN_ASPECT_RATIO: f32 = SCREEN_WIDTH as f32 / SCREEN_HEIGHT as f32;

new_key_type! { pub struct ModelId; }

#[derive(Default)]
pub struct CommandBufferBuilder {
    commands: Vec<Command>,
}

pub struct CommandBuffer {
    commands: Vec<Command>,
}

impl CommandBufferBuilder {

    pub fn build(self) -> CommandBuffer {
        CommandBuffer { commands: self.commands }
    }
    
    pub fn print(&mut self, p: IVec2, text: &str, font_id: FontId) -> &mut Self {
        self.commands.push(Command::Print { p, text: String::from(text), font_id });
        self
    }
    pub fn draw_line(&mut self, p0: IVec2, p1: IVec2) -> &mut Self {
        self.commands.push(Command::DrawLine { p0, p1 });
        self
    }
    pub fn draw_vline(&mut self, x: i32, y0: i32, y1: i32) -> &mut Self {
        self.commands.push(Command::DrawVLine { x, y0, y1 });
        self
    }
    pub fn draw_hline(&mut self, y: i32, x0: i32, x1: i32) -> &mut Self {
        self.commands.push(Command::DrawHLine { y, x0, x1 });
        self
    }
    pub fn draw_rect(&mut self, rect: IRect) -> &mut Self {
        self.commands.push(Command::DrawRect { rect });
        self
    }
    pub fn fill_rect(&mut self, rect: IRect) -> &mut Self {
        self.commands.push(Command::FillRect { rect });
        self
    }
}

impl CommandBuffer {

    pub fn builder() -> CommandBufferBuilder {
        Default::default()
    }
    pub fn build_with<F: Fn(&mut CommandBufferBuilder) -> &mut CommandBufferBuilder>(f: F) -> CommandBuffer {
        let mut builder = Self::builder();
        f(&mut builder);
        builder.build()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.commands.iter()
    }
}