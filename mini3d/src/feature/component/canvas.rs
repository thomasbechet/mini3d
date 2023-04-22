use glam::UVec2;
use serde::{Serialize, Deserialize};
use serde_json::json;

use crate::{renderer::{graphics::Graphics, color::Color, backend::SceneCanvasHandle}, uid::UID, registry::component::{Component, EntityResolver, ComponentDefinition, ComponentProperty}};

#[derive(Serialize, Deserialize)]
pub struct Canvas {
    pub resolution: UVec2,
    pub clear_color: Color,
    pub graphics: Graphics,
    pub visible: bool,
    #[serde(skip)]
    pub(crate) handle: Option<SceneCanvasHandle>,
}

impl Component for Canvas {}

impl Canvas {
    pub const NAME: &'static str = "canvas";
    pub const UID: UID = UID::new(Canvas::NAME);
}