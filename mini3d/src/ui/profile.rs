use glam::Vec2;
use serde::{Serialize, Deserialize};

use crate::{uid::UID, math::rect::IRect, renderer::graphics::Graphics};

#[derive(Serialize, Deserialize)]
pub struct InteractionInputs {

    // Control inputs
    pub click: UID,
    pub scroll: UID,

    // Selection inputs
    pub up: UID,
    pub down: UID,
    pub left: UID,
    pub right: UID,

    // Cursor inputs
    pub cursor_x: UID,
    pub cursor_y: UID,
    pub cursor_motion_x: UID,
    pub cursor_motion_y: UID,
}

#[derive(Serialize, Deserialize)]
struct VisualSelection {
    source_extent: IRect,
    target_extent: IRect,
    source_time: f64,
}

impl VisualSelection {
    fn new(extent: IRect) -> Self {
        Self { source_extent: extent, target_extent: extent, source_time: 0.0 }
    }
}

#[derive(Serialize, Deserialize)]
enum InteractionMode {
    Selection { 
        visual: VisualSelection,
        focus: Option<Vec<UID>>,
    },
    Cursor { position: Vec2 },
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Profile {
    mode: InteractionMode,
    inputs: InteractionInputs,
}

impl Profile {

    pub fn draw(&self, gfx: &mut Graphics, time: f64) {

    }
}