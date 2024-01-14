use core::any::Any;

use alloc::vec::Vec;

use crate::{
    bitset::Bitset, component::Component, context::Context, entity::Entity, error::ComponentError,
};

use super::Container;

pub(crate) struct NativeSingleContainer<C: Component> {
    data: Vec<(Entity, C)>,
    bitset: Bitset,
}

impl<C: Component> NativeSingleContainer<C> {
    pub(crate) fn with_capacity(capacity: u16) -> Self {
        Self {
            data: Vec::with_capacity(capacity as usize),
            bitset: Bitset::with_capacity(capacity),
        }
    }

    pub(crate) fn add(
        &mut self,
        ctx: &mut Context,
        entity: Entity,
        mut component: C,
    ) -> Result<(), ComponentError> {
        let index = entity.index();
        if let Some((e, data)) = self.data.get_mut(index as usize) {
            if *e == Entity::null() {
                component.on_added(entity, ctx)?;
                *e = entity;
                *data = component;
                self.bitset.set(index, true);
            }
        }
        Ok(())
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
}

impl<C: Component> Container for NativeSingleContainer<C> {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn bitset(&self) -> &Bitset {
        &self.bitset
    }

    fn remove(&mut self, ctx: &mut Context, entity: Entity) -> Result<(), ComponentError> {
        let index = entity.index();
        if let Some((e, data)) = self.data.get_mut(index as usize) {
            if *e == entity {
                data.on_removed(entity, ctx)?;
                *e = Entity::null();
                self.bitset.set(index, false);
            }
        }
        Ok(())
    }
}
