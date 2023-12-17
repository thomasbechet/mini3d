use crate::{
    math::vec::V3I32F16,
    renderer::{
        provider::RendererProviderHandle,
        resource::{RenderCameraHandle, TextureHandle},
    },
};

pub enum TextureRenderTarget {
    Texture(TextureHandle),
    CubeMap(TextureHandle),
}

pub enum DiffusePassCommand {
    DrawMesh {
        mesh: RendererProviderHandle,
        material: RendererProviderHandle,
        transform: RendererProviderHandle,
    },
    DrawMeshSkinned {
        mesh: RendererProviderHandle,
        material: RendererProviderHandle,
        transform: RendererProviderHandle,
    },
    DrawBillboard,
    PushPointLight {
        transform: RendererProviderHandle,
        color: V3I32F16,
    },
}

pub struct DiffusePassRenderInfo {
    pub camera: RenderCameraHandle,
    pub target: TextureRenderTarget,
}

pub struct DiffusePassInfo {
    pub per_vertex_lighting: bool,
    pub max_point_lights: u8,
    pub max_spot_lights: u8,
    pub max_directional_lights: u8,
}
