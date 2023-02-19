use std::any::Any;

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize};

use crate::{feature::asset::runtime_component::FieldValue, registry::component::Component};

use std::cell::RefCell;

use super::entity::Entity;

pub(crate) trait AnyComponentContainer {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn entity(&self, index: usize) -> Entity;
    fn contains(&self, entity: Entity) -> bool;
    fn len(&self) -> usize;
    fn remove(&mut self, entity: Entity);
}

pub(crate) struct ComponentContainer<C: Component> {
    pub(crate) components: RefCell<Vec<C>>,
    pub(crate) entities: Vec<Entity>,
    pub(crate) indices: Vec<usize>, // TODO: page optimization
}

impl<C: Component> ComponentContainer<C> {

    pub(crate) fn new() -> Self {
        let mut indices = Vec::with_capacity(500);
        unsafe { indices.set_len(500); }
        Self {
            components: RefCell::new(Vec::with_capacity(128)),
            entities: Vec::with_capacity(128),
            indices,
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entities.len()
    }

    pub(crate) fn capacity(&self) -> usize {
        self.entities.capacity()
    }

    pub(crate) fn contains(&mut self, entity: Entity) -> bool {
        self.entities[entity.index()] == entity
    }

    pub(crate) fn add(&mut self, entity: Entity, component: C) -> Result<()> {
        self.indices[entity.index()] = self.entities.len() - 1;
        self.entities.push(entity);
        self.components
            .try_borrow_mut().with_context(|| "Container already borrowed")?
            .push(component);
        Ok(())    
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> Result<()> {
        let index = self.indices[entity.index()] as usize;
        self.components
            .try_borrow_mut().with_context(|| "Component container already borrowed")?
            .swap_remove(index);
        self.entities.swap_remove(index);
        let swapped_entity = self.entities[index];
        self.indices[swapped_entity.index()] = index;
        self.entities[entity.index()] = Entity::null();
        Ok(())
    }
}

impl<C: Component> AnyComponentContainer for ComponentContainer<C> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) { self }
    fn entity(&self, index: usize) -> Entity { self.entities[index] }
    fn contains(&self, entity: Entity) -> bool { self.contains(entity) }
    fn len(&self) -> usize { self.len() }
    fn remove(&mut self, entity: Entity) { self.remove(entity); }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RuntimeComponent1([FieldValue; 1]);
impl Component for RuntimeComponent1 {}
#[derive(Serialize, Deserialize)]
pub(crate) struct RuntimeComponent2([FieldValue; 2]);
impl Component for RuntimeComponent2 {}
#[derive(Serialize, Deserialize)]
pub(crate) struct RuntimeComponent3([FieldValue; 3]);
impl Component for RuntimeComponent3 {}
#[derive(Serialize, Deserialize)]
pub(crate) struct RuntimeComponent4([FieldValue; 4]);
impl Component for RuntimeComponent4 {}
#[derive(Serialize, Deserialize)]
pub(crate) struct RuntimeComponent5([FieldValue; 5]);
impl Component for RuntimeComponent5 {}
