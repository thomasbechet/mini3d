use anyhow::Result;
use glam::{Vec3, Vec2, Vec4};
use serde::{Serialize, Deserialize};
use slotmap::{new_key_type, Key};

use crate::backend::renderer::{RendererMeshId, RendererBackend, RendererMeshDescriptor};

use super::Asset;

#[derive(Serialize, Deserialize)]
pub struct Vertex {
    pub position: Vec3,
    pub uv: Vec2,
    pub normal: Vec3,
    pub tangent: Vec4, // w: handedness of the tangent space
}

#[derive(Serialize, Deserialize)]
pub struct SubMesh {
    pub vertices: Vec<Vertex>,
}

new_key_type! { pub struct MeshId; }

#[derive(Default, Serialize, Deserialize)]
pub struct Mesh {
    pub submeshes: Vec<SubMesh>,
}

impl Mesh {
    pub fn submit(&mut self, renderer: &mut dyn RendererBackend) -> Result<()> {
        self.renderer_id = renderer.add_mesh(&RendererMeshDescriptor {
            submeshes: &self.submeshes,
        })?;
        Ok(())
    }
    pub fn release(&mut self, renderer: &mut dyn RendererBackend) -> Result<()> {
        renderer.remove_mesh(self.renderer_id)?;
        self.renderer_id = RendererMeshId::null();
        Ok(())
    }
}

impl Asset for Mesh {
    type Id = MeshId;
    fn typename() -> &'static str { "mesh" }
}