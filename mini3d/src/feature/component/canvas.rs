use glam::UVec2;
use mini3d_derive::Component;

use crate::{renderer::{graphics::Graphics, color::Color, backend::SceneCanvasHandle}};

#[derive(Default, Component)]
#[component(name = "canvas")]
pub struct Canvas {
    pub resolution: UVec2,
    pub clear_color: Color,
    pub graphics: Graphics,
    pub visible: bool,
    #[serialize(skip)]
    pub(crate) handle: Option<SceneCanvasHandle>,
}