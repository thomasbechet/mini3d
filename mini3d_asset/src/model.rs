use mini3d_derive::Serialize;
use mini3d_utils::handle::Handle;

pub type ModelHandle = Handle<Model>;

#[derive(Default, Clone, Serialize)]
pub struct Model {
    // pub mesh: MeshHandle,
    // pub materials: Vec<MaterialHandle>,
}

impl Model {}
