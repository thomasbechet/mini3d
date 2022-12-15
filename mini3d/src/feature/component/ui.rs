use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{ui::UI, renderer::backend::{SurfaceCanvasHandle, SceneCanvasHandle}};

#[derive(Serialize, Deserialize)]
pub struct UIComponent {
    pub ui: UI,
    pub position: IVec2,
    #[serde(skip)]
    pub handle: Option<SurfaceCanvasHandle>,
}

impl UIComponent {
    pub fn new(ui: UI, position: IVec2) -> Self {
        Self { ui, position, handle: None }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SceneUIComponent {
    pub ui: UI,
    #[serde(skip)]
    pub handle: Option<SceneCanvasHandle>,
}