use alloc::vec::Vec;
use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        component::{Component, ComponentError, ComponentStorage},
        context::Context,
        entity::Entity,
    },
    math::vec::{V2I32F16, V3I32F16, V4I32F16},
    renderer::provider::RendererProviderHandle,
};

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

#[derive(Default, Clone, Serialize, Reflect)]
pub(crate) struct MeshData {
    pub(crate) submeshes: Vec<SubMesh>,
}

#[derive(Default, Clone, Serialize, Reflect)]
pub struct Mesh {
    pub(crate) data: MeshData,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl Mesh {}

impl Component for Mesh {
    const STORAGE: ComponentStorage = ComponentStorage::Single;
    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        self.handle = ctx.renderer.add_mesh(entity, &self.data)?;
        Ok(())
    }
    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        ctx.renderer.remove_mesh(self.handle)?;
        Ok(())
    }
}
