use mini3d_derive::Serialize;
use mini3d_math::mat::M4I32F16;

use crate::provider::RendererProviderHandle;

#[derive(Clone, Serialize, Default)]
pub struct RenderTransform {
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl RenderTransform {
    pub fn update(value: M4I32F16, teleport: bool) {
        todo!()
    }

    pub fn bind_axis(&mut self, x_axis: &InputAxis, y_axis: &InputAxis) {
        todo!()
    }
}
