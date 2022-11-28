use glam::IVec2;

use crate::{math::rect::IRect, uid::UID};

#[derive(Clone)]
pub enum Command {
    Print { p: IVec2, text: String, font: UID },
    DrawLine { p0: IVec2, p1: IVec2 },
    DrawVLine { x: i32, y0: i32, y1: i32 },
    DrawHLine { y: i32, x0: i32, x1: i32 },
    DrawRect { rect: IRect },
    FillRect { rect: IRect },
    RenderScene { handle: UID },
}

#[derive(Clone)]
pub struct CommandBuffer(Vec<Command>);

impl CommandBuffer {
    
    pub fn empty() -> Self {
        Self(Default::default())
    }

    pub fn push(&mut self, command: Command) {
        self.0.push(command)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Command> {
        self.0.iter()
    }
}
