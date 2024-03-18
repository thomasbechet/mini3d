use mini3d_db::slot_map_key_handle;
use mini3d_derive::Serialize;
use mini3d_math::mat::M4I32F16;

slot_map_key_handle!(RenderTransformHandle);

#[derive(Clone, Serialize, Default)]
pub struct RenderTransformData {
    pub(crate) matrix: M4I32F16
}
