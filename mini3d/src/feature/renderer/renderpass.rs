use glam::Mat4;
use mini3d_derive::{Reflect, Serialize};

use crate::{define_resource_handle, feature::core::resource::Resource, renderer::color::Color};

use super::{
    camera::RenderCameraHandle, material::MaterialHandle, mesh::MeshHandle, texture::TextureHandle,
    transform::RenderTransformHandle,
};

#[derive(Default, Reflect, Serialize)]
pub(crate) enum RenderPassType {
    #[default]
    Unlit,
    Diffuse,
    Reflective,
    Transparent,
    Wireframe,
    Shadow,
    Depth,
    Canvas,
}

#[derive(Default, Reflect, Serialize)]
pub struct RenderPass {
    pub(crate) ty: RenderPassType,
}

impl RenderPass {
    pub const NAME: &'static str = "RTY_RenderPass";
}

impl Resource for RenderPass {}

define_resource_handle!(ForwardPassHandle);
define_resource_handle!(ShadowPassHandle);
define_resource_handle!(GeometryPassHandle);
define_resource_handle!(DeferredPassHandle);

pub enum TextureRenderTarget {
    Texture(TextureHandle),
    CubeMap(TextureHandle),
}

pub enum CullMode {
    None,
    Front,
    Back,
}

/////////////// FORWARD PASS ///////////////

pub enum LightMode {
    Unlit,
    Phong,
    Lambert,
    HalfLambert,
}

pub struct BasicPassInfo {
    pub camera: RenderCameraHandle,
    pub target: TextureRenderTarget,
}

pub struct BasicPassState {
    pub cull_mode: CullMode,
    pub light_mode: LightMode,
}

pub enum BasicPassCommand {
    DrawMesh {
        mesh: MeshHandle,
        material: MaterialHandle,
        state: BasicPassState,
        transform: RenderTransformHandle,
    },
    DrawMeshSkinned {
        mesh: MeshHandle,
        material: MaterialHandle,
        state: BasicPassState,
        transform: RenderTransformHandle,
    },
    DrawBillboard,
}

pub(crate) struct BasicPass {}

/////////////// TRANSPARENT PASS ///////////////

pub struct TransparentPassState {
    pub cull_mode: CullMode,
    pub light_mode: LightMode,
}

pub(crate) struct TransparentPass {}

/////////////// WIREFRAME PASS ///////////////

pub(crate) enum WireframeCommand {
    DrawMesh { mesh: MeshHandle, color: Color },
}

pub(crate) struct WireframePass {}

/////////////// DEPTH PRE PASS ///////////////

pub(crate) struct DepthPrePass {}

/////////////// SHADOW PASS ///////////////

pub struct ShadowPassInfo {
    pub camera: RenderCameraHandle,
    pub view_projection: Mat4,
    pub target: TextureHandle,
}

pub(crate) struct ShadowPass {}

/////////////// CANVAS PASS ///////////////
