use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::renderer::backend::{ViewportHandle, CanvasViewportHandle};

#[derive(Serialize, Deserialize)]
pub struct Viewport {
    pub(crate) pos: IVec2,
    // pub(crate) viewport: Option<ViewportHandle>,
    #[serde(skip)]
    pub(crate) handle: Option<CanvasViewportHandle>,
}