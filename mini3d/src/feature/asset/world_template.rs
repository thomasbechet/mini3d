use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};
use serde_json::{Map, Value};

use crate::{scene::world::World, uid::UID, registry::component::ComponentRegistry};

#[derive(Clone, Serialize, Deserialize)]
pub struct WorldTemplate {
    entities: Map<String, Value>,
}

impl WorldTemplate {
    
    pub fn instantiate(&self, world: &mut World, registry: &ComponentRegistry) -> Result<()> {
        for (_name, components) in &self.entities {
            let components = components.as_object().with_context(|| "Entity components must be an object")?;
            let entity = world.create(); // TODO: Add name to entity
            for (name, data) in components {
                let uid: UID = name.into();
                // let component = &registry.get(uid)
                //     .with_context(|| format!("Component not registered: {}", name))?
                //     .
                // component.instantiate(entity, data, world)?;
            }
        }
        Ok(())
    }
}