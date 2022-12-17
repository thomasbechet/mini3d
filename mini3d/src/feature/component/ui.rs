use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{ui::UI, renderer::backend::{SurfaceCanvasHandle, SceneCanvasHandle}};

#[derive(Serialize, Deserialize)]
pub struct UIComponent {
    pub ui: UI,
    pub position: IVec2,
    pub z_index: i32,
    #[serde(skip)]
    pub handle: Option<SurfaceCanvasHandle>,
}

impl UIComponent {
    pub fn new(ui: UI, position: IVec2, z_index: i32) -> Self {
        Self { ui, position, z_index, handle: None }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SceneUIComponent {
    pub ui: UI,
    #[serde(skip)]
    pub handle: Option<SceneCanvasHandle>,
}