use std::collections::HashMap;

use anyhow::Result;

use super::{entity::Entity, component::Component};

#[derive(Default)]
pub struct World {
    world: hecs::World,
    tags: HashMap<String, hecs::Entity>,
}

impl World {

    pub fn add_entity(&mut self) -> Entity {
        Entity::new(self.world.spawn(()))
    }
    pub fn remove_entity(&mut self, entity: Entity) -> Result<()> {
        self.world.despawn(entity.handle());
        Ok(())
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<()> {
        self.world.insert_one(entity.handle(), component)?;
        Ok(())
    }
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<()> {
        self.world.remove_one::<T>(entity.handle())?;
        Ok(())
    }

    
}