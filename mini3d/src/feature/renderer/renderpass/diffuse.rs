use alloc::vec::Vec;
use glam::Vec3;

use crate::{
    define_resource_handle,
    feature::renderer::{
        camera::RenderCameraHandle, material::MaterialHandle, mesh::MeshHandle,
        texture::TextureHandle, transform::RenderTransformHandle,
    },
};

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
    position: Vec3,
}

pub(crate) struct DiffusePass {
    per_vertex_lighting: bool,
    max_point_lights: u8,
    max_spot_lights: u8,
    max_directional_lights: u8,
    commands: Vec<DiffusePassCommand>,
}

define_resource_handle!(DiffusePassHandle);
