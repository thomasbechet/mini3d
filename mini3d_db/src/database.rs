use crate::{
    container::{ComponentId, ComponentTable},
    entity::Entity,
    error::ComponentError,
    field::{ComponentField, Field, FieldType},
    query::{EntityQuery, Query},
    registry::Registry,
};

#[derive(Default)]
pub struct Database {
    pub(crate) containers: ComponentTable,
    pub(crate) registry: Registry,
}

impl Database {
    pub fn register_tag(&mut self, name: &str) -> Result<ComponentId, ComponentError> {
        let id = self.containers.register_tag(name)?;
        self.registry.add_bitset(id);
        Ok(id)
    }

    pub fn register(
        &mut self,
        name: &str,
        fields: &[ComponentField],
    ) -> Result<ComponentId, ComponentError> {
        let id = self.containers.register(name, fields)?;
        self.registry.add_bitset(id);
        Ok(id)
    }

    pub fn unregister(&mut self, c: ComponentId) {
        self.containers.entries.remove(c);
        self.registry.remove_bitset(c);
    }

    pub fn create(&mut self) -> Entity {
        self.registry.create()
    }

    pub fn destroy(&mut self, e: Entity) {
        self.registry.destroy(e);
        self.containers.remove_all(e);
    }

    pub fn add_default(&mut self, e: Entity, c: ComponentId) {
        self.containers.add_default(e, c);
        self.registry.set(e, c);
    }

    pub fn remove(&mut self, e: Entity, c: ComponentId) {
        self.containers.remove(e, c);
        self.registry.unset(e, c);
    }

    pub fn has(&self, e: Entity, c: ComponentId) -> bool {
        self.registry.has(e, c)
    }

    pub fn read<T: FieldType>(&self, e: Entity, f: Field<T>) -> Option<T> {
        self.containers.read::<T>(e, f.0, f.1)
    }

    pub fn write<T: FieldType>(&mut self, e: Entity, f: Field<T>, v: T) {
        self.containers.write::<T>(e, f.0, f.1, v);
    }

    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.registry.entities()
    }

    pub fn find_component(&self, name: &str) -> Option<ComponentId> {
        self.containers.find_component(name)
    }

    pub fn find_field<T: FieldType>(&self, c: ComponentId, name: &str) -> Option<Field<T>> {
        self.containers
            .find_field(c, name)
            .map(|f| Field(c, f, Default::default()))
    }

    pub fn query_entities<'a>(&self, query: &'a Query) -> EntityQuery<'a> {
        EntityQuery::new(query, self)
    }
}
