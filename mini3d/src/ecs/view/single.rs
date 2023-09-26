use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::{
    ecs::{
        container::single::{AnySingleContainer, StaticSingleContainer},
        entity::Entity,
    },
    registry::component::ComponentData,
};

pub trait StaticSingleView<C: ComponentData> {
    fn get(&self, entity: Entity) -> Option<&C>;
}

pub struct StaticSingleViewRef<'a, C: ComponentData> {
    pub(crate) container: Ref<'a, StaticSingleContainer<C>>,
}

impl<'a, C: ComponentData> StaticSingleViewRef<'a, C> {
    pub fn iter(&self) -> impl Iterator<Item = &C> {
        self.container.iter()
    }

    pub fn singleton(self) -> Option<StaticSingleSingletonRef<'a, C>> {
        // TODO: Ensure has at lease one entity
        Some(StaticSingleSingletonRef(self.container))
    }
}

impl<'a, C: ComponentData> StaticSingleView<C> for StaticSingleViewRef<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.container.get(entity)
    }
}

impl<'a, C: ComponentData> Index<Entity> for StaticSingleViewRef<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

pub struct StaticSingleSingletonRef<'a, C: ComponentData>(Ref<'a, StaticSingleContainer<C>>);

impl<'a, C: ComponentData> Deref for StaticSingleSingletonRef<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.0.iter().next().unwrap()
    }
}

pub struct StaticSingleViewMut<'a, C: ComponentData> {
    pub(crate) container: RefMut<'a, StaticSingleContainer<C>>,
    pub(crate) cycle: u32,
}

impl<'a, C: ComponentData> StaticSingleViewMut<'a, C> {
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.container.get_mut(entity, self.cycle)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut C> {
        self.container.iter_mut()
    }

    pub fn singleton(self) -> Option<StaticSingleSingletonMut<'a, C>> {
        // TODO: Ensure has at lease one entity
        Some(StaticSingleSingletonMut {
            container: self.container,
            cycle: self.cycle,
        })
    }
}

impl<'a, C: ComponentData> StaticSingleView<C> for StaticSingleViewMut<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.container.get(entity)
    }
}

impl<'a, C: ComponentData> Index<Entity> for StaticSingleViewMut<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: ComponentData> IndexMut<Entity> for StaticSingleViewMut<'a, C> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}

pub struct StaticSingleSingletonMut<'a, C: ComponentData> {
    container: RefMut<'a, StaticSingleContainer<C>>,
    cycle: u32,
}

impl<'a, C: ComponentData> Deref for StaticSingleSingletonMut<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.container.iter().next().unwrap()
    }
}

impl<'a, C: ComponentData> DerefMut for StaticSingleSingletonMut<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.container.iter_mut().next().unwrap()
    }
}

pub struct SingleViewRef<'a> {
    pub(crate) container: Ref<'a, Box<dyn AnySingleContainer>>,
}

pub struct SingleViewMut<'a> {
    pub(crate) container: RefMut<'a, Box<dyn AnySingleContainer>>,
    pub(crate) cycle: u32,
}
