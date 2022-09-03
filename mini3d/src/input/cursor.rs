use glam::{Vec2, IVec2};

use crate::graphics::SCREEN_RESOLUTION;

#[derive(Clone, Copy)]
pub struct Cursor {
    position: Vec2,
    motion: Vec2,
}

impl Default for Cursor {
    fn default() -> Self {
        Self { position: Vec2::ZERO, motion: Vec2::ZERO }
    }
}

impl Cursor {
    pub fn screen_position(&self) -> IVec2 {
        self.position.as_ivec2()
    }

    pub fn motion(&self) -> Vec2 {
        self.motion
    }

    pub fn reset_motion(&mut self) {
        self.motion = Vec2::ZERO;
    }

    pub fn translate(&mut self, motion: Vec2) {
        self.motion += motion;
        self.position += motion;
        self.position.clamp(Vec2::ZERO, SCREEN_RESOLUTION.as_vec2());
    }

    pub fn set_position(&mut self, p: Vec2) {
        self.motion += p - self.position;
        self.position = p;
    }
}