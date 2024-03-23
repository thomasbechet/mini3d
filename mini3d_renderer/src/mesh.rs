use alloc::vec::Vec;
use mini3d_derive::Serialize;
use mini3d_math::vec::{V2I32F16, V3I32F16, V4I32F16};
use mini3d_utils::slot_map_key;

slot_map_key!(MeshId);

#[derive(Clone, Serialize)]
pub struct Vertex {
    pub position: V3I32F16,
    pub uv: V2I32F16,
    pub normal: V3I32F16,
    #[serialize(skip)]
    pub tangent: V4I32F16, // w: handedness of the tangent space
}

#[derive(Clone, Serialize)]
pub struct SubMesh {
    pub vertices: Vec<Vertex>,
}

#[derive(Default, Clone, Serialize)]
pub struct MeshData {
    pub(crate) submeshes: Vec<SubMesh>,
}
