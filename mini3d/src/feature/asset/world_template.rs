use std::collections::HashMap;

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};

use crate::{scene::{world::World, component::ComponentEntry}, uid::UID};

#[derive(Clone, Serialize, Deserialize)]
pub struct WorldTemplate {
    entities: Map<String, Value>,
}

impl WorldTemplate {
    
    pub fn instantiate(&self, world: &mut World, component_register: &HashMap<UID, ComponentEntry>) -> Result<()> {
        for (_name, components) in &self.entities {
            let components = components.as_object().with_context(|| "Entity components must be an object")?;
            let entity = world.add_entity(); // TODO: Add name to entity
            for (name, data) in components {
                let uid: UID = name.into();
                let component = &component_register.get(&uid)
                    .with_context(|| format!("Component not registered: {}", name))?
                    .component;
                component.instantiate(entity, data, world)?;
            }
        }
        Ok(())
    }
}