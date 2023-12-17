use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    renderer::provider::RendererProviderHandle,
    resource::{handle::ResourceHandle, Resource, ResourceHookContext},
};

pub mod canvas;
pub mod depth;
pub mod diffuse;
pub mod reflective;
pub mod shadow;
pub mod transparent;
pub mod unlit;
pub mod wireframe;

pub enum CullMode {
    None,
    Front,
    Back,
}

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

// Simple wrappers around ResourceHandle
// There is only one RenderPass type
define_resource_handle!(UnlitPassHandle);
define_resource_handle!(DiffusePassHandle);
define_resource_handle!(ReflectivePassHandle);
define_resource_handle!(TransparentPassHandle);
define_resource_handle!(WireframePassHandle);
define_resource_handle!(ShadowPassHandle);
define_resource_handle!(DepthPassHandle);
define_resource_handle!(CanvasPassHandle);

#[derive(Default, Reflect, Serialize)]
pub struct RenderPass {
    pub(crate) ty: RenderPassType,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl RenderPass {
    pub const NAME: &'static str = "RTY_RenderPass";
}

impl Resource for RenderPass {
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {}
    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {}
}
