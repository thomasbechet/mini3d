use mini3d_db::slot_map_key_handle;

use self::diffuse::DiffusePass;

// pub mod canvas;
// pub mod depth;
pub mod diffuse;
// pub mod reflective;
// pub mod shadow;
// pub mod transparent;
pub mod unlit;
// pub mod wireframe;

slot_map_key_handle!(RenderPassHandle);

pub enum CullMode {
    None,
    Front,
    Back,
}

pub(crate) enum RenderPassType {
    Unlit,
    Diffuse,
    Reflective,
    Transparent,
    Wireframe,
    Shadow,
    Depth,
    Canvas,
}

#[derive(Default)]
pub(crate) enum RenderPassData {
    #[default]
    Unknown,
    Diffuse(DiffusePass),
}
