use glam::{Vec2, Vec3, Vec4};
use mini3d_derive::{Asset, Reflect, Serialize};

#[derive(Clone, Serialize)]
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub normal: Vec3,
    #[serialize(skip)]
    pub tangent: Vec4, // w: handedness of the tangent space
}

#[derive(Clone, Serialize)]
pub struct SubMesh {
    pub vertices: Vec<Vertex>,
}

#[derive(Default, Clone, Asset, Serialize, Reflect)]
pub struct Mesh {
    pub submeshes: Vec<SubMesh>,
}
