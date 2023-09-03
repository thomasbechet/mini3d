use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::registry::component::ComponentData;

use super::{
    component::{AnyComponentContainer, StaticComponentContainer},
    entity::Entity,
};

pub trait StaticComponentView<C: ComponentData> {
    fn get(&self, entity: Entity) -> Option<&C>;
}

pub struct StaticComponentViewRef<'a, C: ComponentData> {
    pub(crate) container: Ref<'a, StaticComponentContainer<C>>,
}

impl<'a, C: ComponentData> StaticComponentViewRef<'a, C> {
    pub fn iter(&self) -> impl Iterator<Item = &C> {
        self.container.iter()
    }

    pub fn singleton(self) -> Option<StaticComponentSingletonRef<'a, C>> {
        // TODO: Ensure has at lease one entity
        Some(StaticComponentSingletonRef(self.container))
    }
}

impl<'a, C: ComponentData> StaticComponentView<C> for StaticComponentViewRef<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.container.get(entity)
    }
}

impl<'a, C: ComponentData> Index<Entity> for StaticComponentViewRef<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

pub struct StaticComponentSingletonRef<'a, C: ComponentData>(Ref<'a, StaticComponentContainer<C>>);

impl<'a, C: ComponentData> Deref for StaticComponentSingletonRef<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.0.iter().next().unwrap()
    }
}

pub struct StaticComponentViewMut<'a, C: ComponentData> {
    pub(crate) container: RefMut<'a, StaticComponentContainer<C>>,
    pub(crate) cycle: u32,
}

impl<'a, C: ComponentData> StaticComponentViewMut<'a, C> {
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.container.get_mut(entity, self.cycle)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut C> {
        self.container.iter_mut()
    }

    pub fn singleton(self) -> Option<StaticComponentSingletonMut<'a, C>> {
        // TODO: Ensure has at lease one entity
        Some(StaticComponentSingletonMut {
            container: self.container,
            cycle: self.cycle,
        })
    }
}

impl<'a, C: ComponentData> StaticComponentView<C> for StaticComponentViewMut<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.container.get(entity)
    }
}

impl<'a, C: ComponentData> Index<Entity> for StaticComponentViewMut<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: ComponentData> IndexMut<Entity> for StaticComponentViewMut<'a, C> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}

pub struct StaticComponentSingletonMut<'a, C: ComponentData> {
    container: RefMut<'a, StaticComponentContainer<C>>,
    cycle: u32,
}

impl<'a, C: ComponentData> Deref for StaticComponentSingletonMut<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.container.iter().next().unwrap()
    }
}

impl<'a, C: ComponentData> DerefMut for StaticComponentSingletonMut<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.container.iter_mut().next().unwrap()
    }
}

pub struct ComponentViewRef<'a> {
    pub(crate) container: Ref<'a, Box<dyn AnyComponentContainer>>,
}

pub struct ComponentViewMut<'a> {
    pub(crate) container: RefMut<'a, Box<dyn AnyComponentContainer>>,
    pub(crate) cycle: u32,
}
