use mini3d_db::slot_map_key_handle;
use mini3d_derive::Serialize;

slot_map_key_handle!(ModelHandle);

#[derive(Default, Clone, Serialize)]
pub struct Model {
    // pub mesh: MeshHandle,
    // pub materials: Vec<MaterialHandle>,
}

impl Model {}
