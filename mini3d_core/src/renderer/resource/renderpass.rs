use mini3d_derive::{Reflect, Serialize};

use crate::resource::Resource;

pub mod canvas;
pub mod depth;
pub mod diffuse;
pub mod reflective;
pub mod shadow;
pub mod transparent;
pub mod unlit;
pub mod wireframe;

#[derive(Default, Reflect, Serialize)]
pub(crate) enum RenderPassType {
    #[default]
    Unlit,
    Diffuse,
    Reflective,
    Transparent,
    Wireframe,
    Shadow,
    Depth,
    Canvas,
}

#[derive(Default, Reflect, Serialize)]
pub struct RenderPass {
    pub(crate) ty: RenderPassType,
}

impl RenderPass {
    pub const NAME: &'static str = "RTY_RenderPass";
}

impl Resource for RenderPass {}

pub enum CullMode {
    None,
    Front,
    Back,
}
