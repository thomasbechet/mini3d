use crate::define_resource_handle;

define_resource_handle!(GraphicsPassHandle);
define_resource_handle!(CanvasPassHandle);
define_resource_handle!(ComputePassHandle);
define_resource_handle!(CopyPassHandle);

pub struct GraphicsPass {
    // color_attachments: Vec<
}

impl GraphicsPass {
    // pub fn with_render_target(mut self, clear) -> Self {

    // }

    // pub fn with_depth_stencil(mut self)
}

pub struct CanvasPass {}

impl CanvasPass {
    pub fn new() -> Self {
        Self {}
    }
}

pub struct ComputePass {}

impl ComputePass {}
