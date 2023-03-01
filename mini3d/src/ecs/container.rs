use std::{any::Any, marker::PhantomData, fmt};

use anyhow::{Result, Context};
use serde::{Serialize, Deserialize, de::{Visitor, self}, Deserializer, Serializer, ser::SerializeTuple};

use crate::{feature::asset::runtime_component::FieldValue};

use std::cell::RefCell;

use super::{entity::Entity, sparse::PagedVector, component::{Component, ComponentRef, ComponentMut}};

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
    pub(crate) indices: PagedVector<usize>,
}

impl<C: Component> ComponentContainer<C> {

    pub(crate) fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_tuple(2)?;
        seq.serialize_element(&self.entities)?;
        seq.serialize_element(&self.components)?;
        seq.end()
    }

    pub(crate) fn deserialize<'a, D: Deserializer<'a>>(deserializer: D) -> Result<Self, D::Error> {
        struct ContainerVisitor<C: Component> { marker: PhantomData<C> }
        impl<'de, C: Component> Visitor<'de> for ContainerVisitor<C> {
            type Value = ComponentContainer<C>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Component container")
            }
            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                where S: de::SeqAccess<'de> {
                use serde::de::Error;
                let components: Vec<C> = seq.next_element()?.with_context(|| "Expect components").map_err(Error::custom)?;
                let entities: Vec<Entity> = seq.next_element()?.with_context(|| "Expect entities").map_err(Error::custom)?;
                let mut container = ComponentContainer::<C> {
                    components: RefCell::new(components),
                    entities,
                    indices: PagedVector::new(),
                };
                for (index, entity) in container.entities.iter().enumerate() {
                    container.indices.set(entity.index(), index);
                }
                Ok(container)
            }
        }
        deserializer.deserialize_tuple(2, ContainerVisitor::<C> { marker: PhantomData })
    }

    pub(crate) fn new() -> Self {
        Self {
            components: RefCell::new(Vec::with_capacity(128)),
            entities: Vec::with_capacity(128),
            indices: PagedVector::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.entities.len()
    }

    pub(crate) fn add(&mut self, entity: Entity, component: C) -> Result<()> {
        self.entities.push(entity);
        self.indices.set(entity.index(), self.entities.len() - 1);
        self.components
            .try_borrow_mut().with_context(|| "Container already borrowed")?
            .push(component);
        Ok(())    
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> Result<()> {
        if let Some(index) = self.indices.get(entity.index()).copied() {
            self.components
                .try_borrow_mut().with_context(|| "Component container already borrowed")?
                .swap_remove(index);
            self.entities.swap_remove(index);
            let swapped_entity = self.entities[index];
            self.indices.set(swapped_entity.index(), index);
            self.entities[entity.index()] = Entity::null();
        }
        Ok(())
    }

    pub(crate) fn get(&self, entity: Entity) -> Option<ComponentRef<'_, C>> {
        let components = self.components.borrow();
        self.indices.get(entity.index()).and_then(|index| {
            if self.entities[*index] == entity {
                Some(ComponentRef { components, index: *index })
            } else {
                None
            }
        })
    }

    pub(crate) fn get_mut(&self, entity: Entity) -> Option<ComponentMut<'_, C>> {
        let components = self.components.borrow_mut();
        self.indices.get(entity.index()).and_then(|index| {
            if self.entities[*index] == entity {
                Some(ComponentMut { components, index: *index })
            } else {
                None
            }
        })
    }
}

impl<C: Component> AnyComponentContainer for ComponentContainer<C> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) { self }
    fn entity(&self, index: usize) -> Entity { self.entities[index] }
    fn contains(&self, entity: Entity) -> bool { self.entities[entity.index()] == entity }
    fn len(&self) -> usize { self.len() }
    fn remove(&mut self, entity: Entity) { self.remove(entity).unwrap(); }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct DynamicComponent1([FieldValue; 1]);
impl Component for DynamicComponent1 {}
#[derive(Serialize, Deserialize)]
pub(crate) struct DynamicComponent2([FieldValue; 2]);
impl Component for DynamicComponent2 {}
#[derive(Serialize, Deserialize)]
pub(crate) struct DynamicComponent3([FieldValue; 3]);
impl Component for DynamicComponent3 {}
#[derive(Serialize, Deserialize)]
pub(crate) struct DynamicComponent4([FieldValue; 4]);
impl Component for DynamicComponent4 {}
#[derive(Serialize, Deserialize)]
pub(crate) struct DynamicComponent5([FieldValue; 5]);
impl Component for DynamicComponent5 {}
