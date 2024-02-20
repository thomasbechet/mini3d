use mini3d_derive::Serialize;
use mini3d_math::vec::V2U32;

use crate::color::Color;

#[derive(Default, Serialize)]
pub struct Canvas {
    pub resolution: V2U32,
    pub clear_color: Color,
    // pub graphics: Graphics,
    pub visible: bool,
}
