use mini3d_derive::{Reflect, Serialize};

use crate::{define_resource_handle, feature::core::resource::Resource};

pub(crate) enum ForwardPassCommand {}

pub(crate) struct ForwardPass {}

pub(crate) struct CanvasPass {}

pub(crate) struct ShadowPass {}

#[derive(Default, Reflect, Serialize)]
pub(crate) enum RenderPassType {
    #[default]
    Forward,
    Shadow,
    Canvas,
    Geometry,
    Deferred,
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
define_resource_handle!(CanvasPassHandle);
define_resource_handle!(GeometryPassHandle);
define_resource_handle!(DeferredPassHandle);
