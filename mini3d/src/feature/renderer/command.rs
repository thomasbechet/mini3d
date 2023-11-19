use crate::define_resource_handle;

pub enum RenderCommandBufferType {
    Graphics,
    Compute,
    Canvas,
}

pub struct RenderCommandBuffer {
    ty: RenderCommandBufferType,
}

impl RenderCommandBuffer {
    pub const NAME: &'static str = "RTY_RenderCommandBuffer";
}

define_resource_handle!(RenderCommandBufferHandle);
