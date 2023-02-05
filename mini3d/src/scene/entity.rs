use std::collections::HashMap;

use anyhow::Result;
use serde::{Serialize, Deserialize};

pub struct EntityResolver {
    map: HashMap<hecs::Entity, hecs::Entity>,
}

pub trait ResolveEntity {
    fn resolve(&mut self, resolver: &EntityResolver) -> Result<()>;
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Entity(pub(crate) hecs::Entity);

impl Entity {
    pub fn resolve(&mut self, resolver: &EntityResolver) {
        if let Some(handle) = resolver.map.get(&self.0) {
            self.0 = *handle;
        }
    }
}