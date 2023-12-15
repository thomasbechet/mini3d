use alloc::vec::Vec;

use crate::{define_resource_handle, math::vec::V3I32F16};

pub enum TextureRenderTarget {
    Texture(TextureHandle),
    CubeMap(TextureHandle),
}

pub(crate) enum DiffusePassCommand {
    DrawMesh {
        mesh: MeshHandle,
        material: MaterialHandle,
        transform: RenderTransformHandle,
    },
    DrawMeshSkinned {
        mesh: MeshHandle,
        material: MaterialHandle,
        transform: RenderTransformHandle,
    },
    DrawBillboard,
}

pub(crate) struct DiffusePassRunInfo {
    pub camera: RenderCameraHandle,
    pub target: TextureRenderTarget,
}

pub(crate) struct PointLight {
    position: V3I32F16,
}

pub(crate) struct DiffusePass {
    per_vertex_lighting: bool,
    max_point_lights: u8,
    max_spot_lights: u8,
    max_directional_lights: u8,
    commands: Vec<DiffusePassCommand>,
}

define_resource_handle!(DiffusePassHandle);
