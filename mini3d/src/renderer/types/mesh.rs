use glam::{Vec2, Vec3, Vec4};
use mini3d_derive::{Reflect, Serialize};

use crate::{
    feature::core::resource::{ResourceData, ResourceHookContext},
    renderer::provider::RendererProviderHandle,
    resource::handle::ResourceHandle,
};

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

#[derive(Default, Clone, Serialize, Reflect)]
pub struct Mesh {
    pub submeshes: Vec<SubMesh>,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl ResourceData for Mesh {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {
        let mesh = ctx.resource.get_mut::<Mesh>(handle).unwrap();
        ctx.renderer.on_mesh_added_hook(mesh, handle);
    }

    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {
        let mesh = ctx.resource.get_mut::<Mesh>(handle).unwrap();
        ctx.renderer.on_mesh_removed_hook(mesh, handle);
    }
}