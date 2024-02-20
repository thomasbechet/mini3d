use mini3d_asset::model::ModelHandle;
use mini3d_derive::Serialize;

use crate::provider::RendererProviderHandle;

#[derive(Default, Serialize, Clone)]
pub struct StaticMesh {
    pub(crate) model: ModelHandle,
    #[serialize(skip)]
    pub(crate) handle: RendererProviderHandle,
}

impl StaticMesh {
    pub fn new(model: ModelHandle) -> Self {
        Self {
            model,
            handle: Default::default(),
        }
    }
}
