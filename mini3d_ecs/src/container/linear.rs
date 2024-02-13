use core::any::Any;

use alloc::vec::Vec;

use crate::{component::NativeComponent, entity::Entity, error::ComponentError};

use super::{Container, NativeContainer};

pub struct LinearContainer<C: NativeComponent> {
    data: Vec<(Entity, C)>,
}

impl<C: NativeComponent> Default for LinearContainer<C> {
    fn default() -> Self {
        Self { data: Vec::new() }
    }
}

impl<C: NativeComponent> LinearContainer<C> {
    pub(crate) fn with_capacity(capacity: u16) -> Self {
        Self {
            data: Vec::with_capacity(capacity as usize),
        }
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

impl<C: NativeComponent + Default> Container for LinearContainer<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl<C: NativeComponent + Default> NativeContainer<C> for LinearContainer<C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.data
            .get(entity.index() as usize)
            .and_then(|(e, data)| if *e == entity { Some(data) } else { None })
    }

    fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.data
            .get_mut(entity.index() as usize)
            .and_then(|(e, data)| if *e == entity { Some(data) } else { None })
    }

    fn add<Context>(
        &mut self,
        entity: Entity,
        component: C,
        ctx: &mut Context,
    ) -> Result<&mut C, ComponentError> {
        let index = entity.index();
        if index >= self.data.len() as u16 {
            self.data
                .resize_with(index as usize + 1, || (Entity::null(), C::default()));
        }
        let (e, data) = &mut self.data[index as usize];
        if *e == Entity::null() {
            *e = entity;
            *data = component;
            (*data).on_added(entity, ctx)?;
            return Ok(data);
        }
        Err(ComponentError::DuplicatedEntry)
    }

    fn remove<Context>(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        let index = entity.index();
        if let Some((e, data)) = self.data.get_mut(index as usize) {
            if *e == entity {
                *e = Entity::null();
                data.on_removed(entity, ctx)?;
                return Ok(());
            }
        }
        Err(ComponentError::EntryNotFound)
    }
}
