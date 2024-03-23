use mini3d_derive::Serialize;
use mini3d_math::mat::M4I32F16;
use mini3d_utils::slot_map_key;

slot_map_key!(RenderTransformId);

#[derive(Clone, Serialize, Default)]
pub struct RenderTransformData {
    pub(crate) matrix: M4I32F16
}
