use std::collections::HashMap;

use mini3d_derive::{Component, Reflect, Serialize};

use crate::ecs::entity::Entity;

#[derive(Serialize)]
pub struct EntityPrefab {}

#[derive(Component, Serialize, Default, Reflect)]
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
