use mini3d_derive::{Reflect, Resource, Serialize};

use crate::renderer::{
    handle::{RendererMaterialHandle, RendererMeshHandle},
    provider::RendererProviderHandle,
};

#[derive(Default, Clone, Resource, Serialize, Reflect)]
pub struct Model {
    pub mesh: RendererMeshHandle,
    pub materials: Vec<RendererMaterialHandle>,
    pub(crate) handle: RendererProviderHandle,
}
