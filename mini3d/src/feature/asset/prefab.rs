use std::collections::HashMap;

use mini3d_derive::{Asset, Serialize};

use crate::ecs::entity::Entity;

#[derive(Serialize)]
pub struct EntityPrefab {}

#[derive(Asset)]
pub struct Prefab {
    pub(crate) entities: HashMap<Entity, EntityPrefab>,
    pub(crate) names: HashMap<String, Entity>,
}

impl Prefab {
    pub fn empty() -> Self {
        Self {
            entities: HashMap::new(),
            names: HashMap::new(),
        }
    }
}
