use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::{registry::asset::Asset, uid::UID, ecs::entity::Entity};

#[derive(Serialize, Deserialize)]
pub struct EntityPrefab {
    components: HashMap<UID, serde_json::Value>, // True serde component serialization
}

#[derive(Serialize, Deserialize)]
pub struct Prefab {
    pub(crate) entities: HashMap<Entity, EntityPrefab>,
    pub(crate) names: HashMap<String, Entity>,
}

impl Asset for Prefab {}

impl Prefab {

    pub const NAME: &'static str = "prefab";
    pub const UID: UID = UID::new(Prefab::NAME);

    pub fn empty() -> Self {
        Self { entities: HashMap::new(), names: HashMap::new() }
    }
}