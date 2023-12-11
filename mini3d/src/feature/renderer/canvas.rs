use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    math::vec::V2U32,
    renderer::{color::Color, provider::RendererProviderHandle},
};

#[derive(Default, Component, Serialize, Reflect)]
pub struct Canvas {
    pub resolution: V2U32,
    pub clear_color: Color,
    // pub graphics: Graphics,
    pub visible: bool,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}
