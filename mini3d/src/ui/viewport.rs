use serde::{Serialize, Deserialize};

use crate::{math::rect::IRect, uid::UID};

#[derive(Serialize, Deserialize)]
pub struct ViewportUI {
    extent: IRect,
    scene: UID,
}