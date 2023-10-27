use crate::resource::handle::ResourceHandle;

pub struct RenderPassHandle(pub(crate) ResourceHandle);

pub struct RenderPass {
    color_attachments: Vec<
}

impl RenderPass {
    pub fn with_render_target(mut self, clear) -> Self {

    }

    pub fn with_depth_stencil(mut self)
}
