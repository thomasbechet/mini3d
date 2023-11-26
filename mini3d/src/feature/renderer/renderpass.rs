use crate::define_resource_handle;

pub(crate) struct ForwardPass {}

pub(crate) struct CanvasPass {}

pub(crate) struct ShadowPass {}

pub(crate) enum RenderPassType {
    Forward,
    Shadow,
    Canvas,
    Geometry,
    Deferred,
}

pub struct RenderPass {
    ty: RenderPassType,
}

impl RenderPass {
    pub const NAME: &'static str = "RTY_RenderPass";
}

define_resource_handle!(ForwardPassHandle);
define_resource_handle!(ShadowPassHandle);
define_resource_handle!(CanvasPassHandle);
define_resource_handle!(GeometryPassHandle);
define_resource_handle!(DeferredPassHandle);
