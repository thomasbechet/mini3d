use glam::{Vec3, Vec2, Vec4};

use super::Asset;

pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub normal: Vec3,
    pub tangent: Vec4, // w: handedness of the tangent space
}

pub struct SubMesh {
    pub vertices: Vec<Vertex>,
}

#[derive(Default)]
pub struct Mesh {
    pub submeshes: Vec<SubMesh>,
}

impl Asset for Mesh {
    fn typename() -> &'static str {
        "mesh"
    }

    fn default() -> Self {
        Default::default()
    }
}