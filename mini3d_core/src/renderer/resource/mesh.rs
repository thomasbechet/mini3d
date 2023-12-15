use alloc::vec::Vec;
use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    math::vec::{V2I32F16, V3I32F16, V4I32F16},
    renderer::provider::RendererProviderHandle,
    resource::{handle::ResourceHandle, Resource, ResourceHookContext},
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
pub struct Mesh {
    pub submeshes: Vec<SubMesh>,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl Mesh {
    pub const NAME: &'static str = "RTY_Mesh";
}

impl Resource for Mesh {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {
        let mesh = ctx.resource.native_mut::<Mesh>(handle).unwrap();
        ctx.renderer.on_mesh_added_hook(mesh, handle.into());
    }

    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {
        let mesh = ctx.resource.native_mut::<Mesh>(handle).unwrap();
        ctx.renderer.on_mesh_removed_hook(mesh, handle.into());
    }
}

define_resource_handle!(MeshHandle);
