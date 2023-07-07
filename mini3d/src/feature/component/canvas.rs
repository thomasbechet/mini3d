use glam::UVec2;
use mini3d_derive::{Component, Reflect, Serialize};

use crate::renderer::{backend::SceneCanvasHandle, color::Color, graphics::Graphics};

#[derive(Default, Component, Serialize, Reflect)]
pub struct Canvas {
    pub resolution: UVec2,
    pub clear_color: Color,
    pub graphics: Graphics,
    pub visible: bool,
    #[serialize(skip)]
    pub(crate) handle: Option<SceneCanvasHandle>,
}
