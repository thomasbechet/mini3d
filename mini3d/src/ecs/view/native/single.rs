use std::ops::{Index, IndexMut};

use crate::{
    api::Context,
    ecs::{
        container::{native::single::NativeSingleContainer, Container},
        entity::Entity,
        error::ResolverError,
        system::SystemResolver,
    },
    feature::ecs::component::Component,
    utils::uid::ToUID,
};

pub trait NativeSingleView<C: Component> {
    fn get(&self, entity: Entity) -> Option<&C>;
}

// Native single reference

pub struct NativeSingleViewRef<C: Component> {
    pub(crate) container: *const NativeSingleContainer<C>,
}

impl<C: Component> Default for NativeSingleViewRef<C> {
    fn default() -> Self {
        Self {
            container: std::ptr::null(),
        }
    }
}

impl<C: Component> NativeSingleViewRef<C> {
    pub fn resolve(
        &mut self,
        resolver: &mut SystemResolver,
        component: impl ToUID,
    ) -> Result<(), ResolverError> {
        let id = resolver.read(component)?;
        self.container = resolver
            .containers
            .entries
            .get(id.0)
            .unwrap()
            .container
            .as_any()
            .downcast_ref::<NativeSingleContainer<C>>()
            .unwrap();
        Ok(())
    }

    pub fn iter(&self) -> impl Iterator<Item = &C> {
        unsafe { &*self.container }.iter()
    }
}

impl<C: Component> NativeSingleView<C> for NativeSingleViewRef<C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        unsafe { &*self.container }.get(entity)
    }
}

impl<C: Component> Index<Entity> for NativeSingleViewRef<C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

// Native single mutable reference

pub struct NativeSingleViewMut<C: Component> {
    pub(crate) container: *mut NativeSingleContainer<C>,
}

impl<C: Component> Default for NativeSingleViewMut<C> {
    fn default() -> Self {
        Self {
            container: std::ptr::null_mut(),
        }
    }
}

impl<C: Component> NativeSingleViewMut<C> {
    pub fn resolve(
        &mut self,
        resolver: &mut SystemResolver,
        component: impl ToUID,
    ) -> Result<(), ResolverError> {
        let id = resolver.write(component)?;
        self.container = resolver
            .containers
            .entries
            .get_mut(id.0)
            .unwrap()
            .container
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<C>>()
            .unwrap();
        Ok(())
    }

    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        unsafe { &mut *self.container }.get_mut(entity)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut C> {
        unsafe { &mut *self.container }.iter_mut()
    }

    pub fn add(&mut self, entity: Entity, component: C) {
        unsafe { &mut *self.container }.add(entity, component);
    }

    pub fn remove(&mut self, ctx: &mut Context, entity: Entity) {
        unsafe { &mut *self.container }.remove(entity)
    }
}

impl<C: Component> NativeSingleView<C> for NativeSingleViewMut<C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        unsafe { &*self.container }.get(entity)
    }
}

impl<C: Component> Index<Entity> for NativeSingleViewMut<C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<C: Component> IndexMut<Entity> for NativeSingleViewMut<C> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}
