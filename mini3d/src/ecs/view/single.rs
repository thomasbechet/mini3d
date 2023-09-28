use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::{
    ecs::{
        container::single::{AnySingleContainer, StaticSingleContainer},
        entity::Entity,
    },
    registry::datatype::StaticDataType,
};

pub trait StaticSingleView<D: StaticDataType> {
    fn get(&self, entity: Entity) -> Option<&D>;
}

pub struct StaticSingleViewRef<'a, D: StaticDataType> {
    pub(crate) container: Ref<'a, StaticSingleContainer<D>>,
}

impl<'a, D: StaticDataType> StaticSingleViewRef<'a, D> {
    pub fn iter(&self) -> impl Iterator<Item = &D> {
        self.container.iter()
    }

    pub fn singleton(self) -> Option<StaticSingleSingletonRef<'a, D>> {
        // TODO: Ensure has at lease one entity
        Some(StaticSingleSingletonRef(self.container))
    }
}

impl<'a, D: StaticDataType> StaticSingleView<D> for StaticSingleViewRef<'a, D> {
    fn get(&self, entity: Entity) -> Option<&D> {
        self.container.get(entity)
    }
}

impl<'a, D: StaticDataType> Index<Entity> for StaticSingleViewRef<'a, D> {
    type Output = D;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

pub struct StaticSingleSingletonRef<'a, D: StaticDataType>(Ref<'a, StaticSingleContainer<D>>);

impl<'a, D: StaticDataType> Deref for StaticSingleSingletonRef<'a, D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        self.0.iter().next().unwrap()
    }
}

pub struct StaticSingleViewMut<'a, D: StaticDataType> {
    pub(crate) container: RefMut<'a, StaticSingleContainer<D>>,
    pub(crate) cycle: u32,
}

impl<'a, D: StaticDataType> StaticSingleViewMut<'a, D> {
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut D> {
        self.container.get_mut(entity, self.cycle)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut D> {
        self.container.iter_mut()
    }

    pub fn singleton(self) -> Option<StaticSingleSingletonMut<'a, D>> {
        // TODO: Ensure has at lease one entity
        Some(StaticSingleSingletonMut {
            container: self.container,
            cycle: self.cycle,
        })
    }
}

impl<'a, D: StaticDataType> StaticSingleView<D> for StaticSingleViewMut<'a, D> {
    fn get(&self, entity: Entity) -> Option<&D> {
        self.container.get(entity)
    }
}

impl<'a, D: StaticDataType> Index<Entity> for StaticSingleViewMut<'a, D> {
    type Output = D;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, D: StaticDataType> IndexMut<Entity> for StaticSingleViewMut<'a, D> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}

pub struct StaticSingleSingletonMut<'a, D: StaticDataType> {
    container: RefMut<'a, StaticSingleContainer<D>>,
    cycle: u32,
}

impl<'a, D: StaticDataType> Deref for StaticSingleSingletonMut<'a, D> {
    type Target = D;

    fn deref(&self) -> &Self::Target {
        self.container.iter().next().unwrap()
    }
}

impl<'a, D: StaticDataType> DerefMut for StaticSingleSingletonMut<'a, D> {
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
