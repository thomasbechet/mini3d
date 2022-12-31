use glam::UVec2;
use serde::{Serialize, Deserialize};

use crate::renderer::{graphics::Graphics, color::Color, backend::SceneCanvasHandle};

#[derive(Serialize, Deserialize)]
pub struct CanvasComponent {
    pub resolution: UVec2,
    pub clear_color: Color,
    pub graphics: Graphics,
    pub visible: bool,
    #[serde(skip)]
    pub(crate) handle: Option<SceneCanvasHandle>,
}