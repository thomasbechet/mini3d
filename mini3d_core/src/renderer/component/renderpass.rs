use mini3d_derive::{Reflect, Serialize};

pub mod canvas;
pub mod depth;
pub mod diffuse;
pub mod reflective;
pub mod shadow;
pub mod transparent;
pub mod unlit;
pub mod wireframe;

pub enum CullMode {
    None,
    Front,
    Back,
}

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
