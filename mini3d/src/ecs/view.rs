use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::registry::component::Component;

use super::{
    component::{AnyComponentContainer, StaticComponentContainer},
    entity::Entity,
    error::SceneError,
};

pub trait StaticComponentView<C: Component> {
    fn get(&self, entity: Entity) -> Option<&C>;
}

pub struct StaticComponentViewRef<'a, C: Component> {
    container: &'a StaticComponentContainer<C>,
}

impl<'a, C: Component> StaticComponentViewRef<'a, C> {
    pub fn iter(&self) -> impl Iterator<Item = &C> {
        self.container.iter()
    }
}

impl<'a, C: Component> StaticComponentView<C> for StaticComponentViewRef<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.container.get(entity)
    }
}

impl<'a, C: Component> Index<Entity> for StaticComponentViewRef<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

pub struct StaticComponentViewMut<'a, C: Component> {
    container: &'a mut StaticComponentContainer<C>,
    cycle: u32,
}

impl<'a, C: Component> StaticComponentViewMut<'a, C> {
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.container.get_mut(entity, self.cycle)
    }
}

impl<'a, C: Component> StaticComponentView<C> for StaticComponentViewMut<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.container.get(entity)
    }
}

impl<'a, C: Component> Index<Entity> for StaticComponentViewMut<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> IndexMut<Entity> for StaticComponentViewMut<'a, C> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}

pub struct ComponentViewRef<'a> {
    pub(crate) container: Ref<'a, Box<dyn AnyComponentContainer>>,
}

pub struct ComponentViewMut<'a> {
    pub(crate) container: RefMut<'a, Box<dyn AnyComponentContainer>>,
    pub(crate) cycle: u32,
}

impl<'a> ComponentViewRef<'a> {
    pub fn as_static<C: Component>(self) -> Result<StaticComponentViewRef<'a, C>, SceneError> {
        Ok(StaticComponentViewRef {
            container: self
                .container
                .as_any()
                .downcast_ref::<StaticComponentContainer<C>>()
                .ok_or(SceneError::ComponentTypeMismatch)?,
        })
    }
}

impl<'a> Deref for ComponentViewRef<'a> {
    type Target = Box<dyn AnyComponentContainer>;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl<'a> ComponentViewMut<'a> {
    pub fn as_static<C: Component>(&self) -> Result<StaticComponentViewMut<'a, C>, SceneError> {
        Ok(StaticComponentViewMut {
            container: self
                .container
                .as_any()
                .downcast_mut::<StaticComponentContainer<C>>()
                .ok_or(SceneError::ComponentTypeMismatch)?,
            cycle: self.cycle,
        })
    }
}

impl<'a> Deref for ComponentViewMut<'a> {
    type Target = Box<dyn AnyComponentContainer>;

    fn deref(&self) -> &Self::Target {
        &self.container
    }
}

impl<'a> DerefMut for ComponentViewMut<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.container
    }
}
