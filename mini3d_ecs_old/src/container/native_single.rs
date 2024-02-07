use core::any::Any;

use alloc::vec::Vec;

use crate::{component::Component, context::Context, entity::Entity, error::ComponentError};

use super::Container;

pub(crate) struct NativeSingleContainer<C: Component> {
    data: Vec<(Entity, C)>,
}

impl<C: Component> NativeSingleContainer<C> {
    pub(crate) fn with_capacity(capacity: u16) -> Self {
        Self {
            data: Vec::with_capacity(capacity as usize),
        }
    }

    pub(crate) fn add(&mut self, entity: Entity, component: C) -> Option<&mut C> {
        let index = entity.index();
        if index >= self.data.len() as u16 {
            self.data
                .resize_with(index as usize + 1, || (Entity::null(), C::default()));
        }
        let (e, data) = &mut self.data[index as usize];
        if *e == Entity::null() {
            *e = entity;
            *data = component;
            self.bitset.set(index, true);
            return Some(data);
        }
        None
    }

    pub(crate) fn remove(&mut self, entity: Entity) -> Option<&mut C> {
        let index = entity.index();
        if let Some((e, data)) = self.data.get_mut(index as usize) {
            if *e == entity {
                *e = Entity::null();
                self.bitset.set(index, false);
                return Some(data);
            }
        }
        None
    }

    pub(crate) fn get(&self, entity: Entity) -> Option<&C> {
        self.data
            .get(entity.index() as usize)
            .and_then(|(e, data)| if *e == entity { Some(data) } else { None })
    }

    pub(crate) fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.data
            .get_mut(entity.index() as usize)
            .and_then(|(e, data)| if *e == entity { Some(data) } else { None })
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (Entity, &C)> {
        self.data.iter().filter_map(|(e, data)| {
            if *e != Entity::null() {
                Some((*e, data))
            } else {
                None
            }
        })
    }

    pub(crate) fn iter_mut(&mut self) -> impl Iterator<Item = (Entity, &mut C)> {
        self.data.iter_mut().filter_map(|(e, data)| {
            if *e != Entity::null() {
                Some((*e, data))
            } else {
                None
            }
        })
    }
}

impl<C: Component> Container for NativeSingleContainer<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn remove(&mut self, ctx: &mut Context, entity: Entity) -> Result<(), ComponentError> {
        if let Some(data) = self.remove(entity) {
            data.on_removed(entity, ctx)?;
        }
        Ok(())
    }
}
