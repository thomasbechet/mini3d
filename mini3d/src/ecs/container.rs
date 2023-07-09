use super::{
    entity::Entity,
    error::ECSError,
    reference::{StaticComponentMut, StaticComponentRef},
    sparse::PagedVector,
    view::{
        AnyComponentViewMut, AnyComponentViewRef, StaticComponentViewMut, StaticComponentViewRef,
    },
};
use crate::{
    registry::component::Component,
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
};
use core::{any::Any, cell::RefCell};

pub(crate) struct StaticComponentContainer<C: Component> {
    pub(crate) components: RefCell<Vec<C>>,
    pub(crate) entities: Vec<Entity>,
    pub(crate) indices: PagedVector<usize>,
}

impl<C: Component> StaticComponentContainer<C> {
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

    pub(crate) fn add(&mut self, entity: Entity, component: C) -> Result<(), ECSError> {
        self.entities.push(entity);
        self.indices.set(entity.key(), self.entities.len() - 1);
        self.components
            .try_borrow_mut()
            .map_err(|_| ECSError::ContainerBorrowMut)?
            .push(component);
        Ok(())
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> Result<(), ECSError> {
        if let Some(index) = self.indices.get(entity.key()).copied() {
            self.components
                .try_borrow_mut()
                .map_err(|_| ECSError::ContainerBorrowMut)?
                .swap_remove(index);
            self.entities.swap_remove(index);
            let swapped_entity = self.entities[index];
            self.indices.set(swapped_entity.key(), index);
            self.entities[entity.key() as usize] = Entity::null();
        }
        Ok(())
    }

    pub(crate) fn get(&self, entity: Entity) -> Option<StaticComponentRef<'_, C>> {
        let components = self.components.borrow();
        self.indices.get(entity.key()).and_then(|index| {
            if self.entities[*index] == entity {
                Some(StaticComponentRef {
                    components,
                    index: *index,
                })
            } else {
                None
            }
        })
    }

    pub(crate) fn get_mut(&self, entity: Entity) -> Option<StaticComponentMut<'_, C>> {
        let components = self.components.borrow_mut();
        self.indices.get(entity.key()).and_then(|index| {
            if self.entities[*index] == entity {
                Some(StaticComponentMut {
                    components,
                    index: *index,
                })
            } else {
                None
            }
        })
    }

    pub(crate) fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        // Write header
        C::Header::default().serialize(encoder)?;
        // Write entity count
        encoder.write_u32(self.entities.len() as u32)?;
        // Write components
        for component in self.components.borrow().iter() {
            component.serialize(encoder)?;
        }
        // Write entities
        for entity in self.entities.iter() {
            encoder.write_u32(entity.key())?;
        }
        Ok(())
    }

    pub(crate) fn deserialize(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        // Reset container
        let mut components = self.components.borrow_mut();
        components.clear();
        self.entities.clear();
        // Read header
        let header = C::Header::deserialize(decoder, &Default::default())?;
        // Read entity count
        let count = decoder.read_u32()?;
        // Read components
        for _ in 0..count {
            let component = C::deserialize(decoder, &header)?;
            components.push(component);
        }
        // Read entities
        for _ in 0..count {
            self.entities.push(Entity(decoder.read_u32()?));
        }
        // Update indices
        for (index, entity) in self.entities.iter().enumerate() {
            self.indices.set(entity.key(), index);
        }
        Ok(())
    }
}

pub(crate) struct DynamicComponentContainer {
    pub(crate) entities: Vec<Entity>,
    pub(crate) indices: PagedVector<usize>,
}

pub(crate) trait AnyComponentContainer {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn entity(&self, index: usize) -> Entity;
    fn contains(&self, entity: Entity) -> bool;
    fn len(&self) -> usize;
    fn remove(&mut self, entity: Entity);
    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError>;
    fn any_view(&self) -> AnyComponentViewRef<'_>;
    fn any_view_mut(&self) -> AnyComponentViewMut<'_>;
}

impl<C: Component> AnyComponentContainer for StaticComponentContainer<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) {
        self
    }
    fn entity(&self, index: usize) -> Entity {
        self.entities[index]
    }
    fn contains(&self, entity: Entity) -> bool {
        if let Some(index) = self.indices.get(entity.key()).copied() {
            index < self.entities.len() && self.entities[index] == entity
        } else {
            false
        }
    }
    fn len(&self) -> usize {
        self.len()
    }
    fn remove(&mut self, entity: Entity) {
        self.remove(entity).unwrap();
    }
    fn serialize(&self, mut encoder: &mut dyn Encoder) -> Result<(), EncoderError> {
        self.serialize(&mut encoder)
    }
    fn deserialize(&mut self, mut decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        self.deserialize(&mut decoder)
    }
    fn any_view(&self) -> AnyComponentViewRef<'_> {
        AnyComponentViewRef {
            view: Box::new(StaticComponentViewRef::new(self)),
        }
    }
    fn any_view_mut(&self) -> AnyComponentViewMut<'_> {
        AnyComponentViewMut {
            view: Box::new(StaticComponentViewMut::new(self)),
        }
    }
}

impl AnyComponentContainer for DynamicComponentContainer {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn entity(&self, index: usize) -> Entity {
        todo!()
    }
    fn contains(&self, entity: Entity) -> bool {
        todo!()
    }
    fn len(&self) -> usize {
        todo!()
    }
    fn remove(&mut self, entity: Entity) {
        todo!()
    }
    fn serialize(&self, encoder: &mut dyn Encoder) -> Result<(), EncoderError> {
        todo!()
    }
    fn deserialize(&mut self, decoder: &mut dyn Decoder) -> Result<(), DecoderError> {
        todo!()
    }
    fn any_view(&self) -> AnyComponentViewRef<'_> {
        todo!()
    }
    fn any_view_mut(&self) -> AnyComponentViewMut<'_> {
        todo!()
    }
}
