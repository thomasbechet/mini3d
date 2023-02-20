use std::{collections::HashMap};

use anyhow::{Context, Result};
use serde::Deserializer;

use crate::{uid::UID, registry::component::{Component, ComponentRegistry}};

use super::{entity::Entity, container::{AnyComponentContainer, ComponentContainer}, view::{ComponentView, ComponentViewMut}, query::Query, sparse::PagedVector};

#[derive(Default, Clone, Copy)]
struct EntityEntry {
    alive: bool,
}

#[derive(Default)]
pub struct World {
    containers: HashMap<UID, Box<dyn AnyComponentContainer>>,
    entities: PagedVector<EntityEntry>,
    free_entities: Vec<Entity>,
}

impl World {

    pub(crate) fn deserialize<'a, D: Deserializer<'a>>(registry: &ComponentRegistry, deserializer: D) -> Result<World, D::Error> {
        
    }

    pub(crate) fn create(&mut self) -> Entity {
        if let Some(entity) = self.free_entities.pop() {
            return entity;
        }
        Entity::null()
    }

    pub(crate) fn destroy(&mut self, entity: Entity) -> Result<()> {
        for container in self.containers.values_mut() {
            container.remove(entity);
        }
        self.free_entities.push(Entity::new(entity.index(), entity.version() + 1));
        Ok(())
    }

    pub(crate) fn add<C: Component>(&mut self, registry: &ComponentRegistry, entity: Entity, component: UID, data: C) -> Result<()> {
        if !self.containers.contains_key(&component) {
            let pool = registry
                .get(component).with_context(|| "Component not registered")?
                .reflection.create_container();
            self.containers.insert(component, pool);
        }
        let pool = self.containers.get_mut(&component).unwrap();
        pool.as_any_mut()
            .downcast_mut::<ComponentContainer<C>>().with_context(|| "Component type mismatch")?
            .add(entity, data);
        Ok(())
    }
    
    pub(crate) fn remove(&mut self, registry: ComponentRegistry, entity: Entity, component: UID) -> Result<()> {
        let pool = self.containers.get_mut(&component).with_context(|| "Component container not found")?;
        pool.remove(entity);
        Ok(())
    }

    pub(crate) fn view<'a, C: Component>(&'a self, component: UID) -> Result<ComponentView<'a, C>> {
        let container = self.containers.get(&component).with_context(|| "Component container not found")?;
        let container = container.as_any()
            .downcast_ref::<ComponentContainer<C>>().with_context(|| "Component type mismatch")?;
        Ok(ComponentView::new(container))
    }

    pub(crate) fn view_mut<'a, C: Component>(&'a self, component: UID) -> Result<ComponentViewMut<'a, C>> {
        let container = self.containers.get(&component).with_context(|| "Component container not found")?;
        let container = container.as_any()
            .downcast_ref::<ComponentContainer<C>>().with_context(|| "Component type mismatch")?;
        Ok(ComponentViewMut::new(container))
    }

    pub(crate) fn query<'a>(&'a self, components: &[UID]) -> Query<'a> {
        let mut containers = Vec::new();
        for component in components {
            containers.push(self.containers.get(component).unwrap().as_ref());
        }
        containers.sort_by(|a, b| a.len().cmp(&b.len()));
        Query::new(containers)
    }
}