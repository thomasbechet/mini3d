use glam::{Vec3, Vec2, Vec4};
use serde::{Serialize, Deserialize};

use super::Asset;

#[derive(Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub normal: Vec3,
    pub tangent: Vec4, // w: handedness of the tangent space
}

#[derive(Clone, Serialize, Deserialize)]
pub struct SubMesh {
    pub vertices: Vec<Vertex>,
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub submeshes: Vec<SubMesh>,
}

impl Asset for Mesh {
    fn typename() -> &'static str { "mesh" }
}