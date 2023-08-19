use mini3d_derive::{Component, Reflect, Serialize};

use crate::{asset::handle::StaticAsset, renderer::backend::SceneModelHandle};

use super::model::Model;

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct StaticMesh {
    pub model: StaticAsset<Model>,
    #[serialize(skip)]
    pub(crate) handle: SceneModelHandle,
}

impl StaticMesh {
    pub fn new(model: StaticAsset<Model>) -> Self {
        Self {
            model,
            handle: Default::default(),
        }
    }
}
