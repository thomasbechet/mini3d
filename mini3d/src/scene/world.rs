use anyhow::Result;

use super::{entity::Entity, component::Component, query::{QueryView, QueryIter}};

#[derive(Default)]
pub struct World {
    pub(crate) raw_world: hecs::World,
}

impl World {

    pub fn add_entity(&mut self) -> Entity {
        Entity(self.raw_world.spawn(()))
    }
    pub fn remove_entity(&mut self, entity: Entity) -> Result<()> {
        self.raw_world.despawn(entity.0);
        Ok(())
    }

    pub fn add_component<T: Component>(&mut self, entity: Entity, component: T) -> Result<()> {
        self.raw_world.insert_one(entity.0, component)?;
        Ok(())
    }
    pub fn remove_component<T: Component>(&mut self, entity: Entity) -> Result<()> {
        self.raw_world.remove_one::<T>(entity.0)?;
        Ok(())
    }
    
    pub fn query_mut<Q: hecs::Query>(&mut self) -> QueryIter<'_, Q> {
        QueryIter(self.raw_world.query_mut::<Q>().into_iter())
    }
    pub fn query<Q: hecs::Query>(&self) -> QueryIter<'_, Q> {
        QueryIter(self.raw_world.query::<Q>().into_iter())
    }
    pub fn query_one_mut<Q: hecs::Query>(&mut self, entity: Entity) -> Result<hecs::QueryItem<'_, Q>> {
        Ok(self.raw_world.query_one_mut::<Q>(entity.0)?)
    }
    pub fn query_one<'a, Q: hecs::Query + 'a>(&'a self, entity: Entity) -> Result<hecs::QueryItem<'a, Q>> {
        Ok(self.raw_world.query_one::<Q>(entity.0)?.get().unwrap())
    }
    pub fn view_mut<Q: hecs::Query>(&mut self) -> QueryView<'_, Q> {
        QueryView(self.raw_world.query_mut::<Q>().view())
    }
    
}