use std::{
    cell::{Ref, RefMut},
    ops::{Deref, Index, IndexMut},
};

use crate::{
    ecs::{
        container::array::{AnyArrayContainer, StaticArrayContainer},
        entity::Entity,
    },
    registry::datatype::StaticDataType,
};

pub trait StaticArrayView<D: StaticDataType> {
    fn get(&self, entity: Entity) -> Option<&[D]>;
}

pub struct StaticArrayViewRef<'a, D: StaticDataType> {
    pub(crate) container: Ref<'a, StaticArrayContainer<D>>,
}

impl<'a, D: StaticDataType> StaticArrayViewRef<'a, D> {
    pub fn iter(&self) -> impl Iterator<Item = &[D]> {
        self.container.iter()
    }
}

impl<'a, D: StaticDataType> StaticArrayView<D> for StaticArrayViewRef<'a, D> {
    fn get(&self, entity: Entity) -> Option<&[D]> {
        self.container.get(entity)
    }
}

impl<'a, D: StaticDataType> Index<Entity> for StaticArrayViewRef<'a, D> {
    type Output = [D];

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

pub struct StaticArrayArraytonRef<'a, D: StaticDataType>(Ref<'a, StaticArrayContainer<D>>);

impl<'a, D: StaticDataType> Deref for StaticArrayArraytonRef<'a, D> {
    type Target = [D];

    fn deref(&self) -> &Self::Target {
        self.0.iter().next().unwrap()
    }
}

pub struct StaticArrayViewMut<'a, D: StaticDataType> {
    pub(crate) container: RefMut<'a, StaticArrayContainer<D>>,
    pub(crate) cycle: u32,
}

impl<'a, D: StaticDataType> StaticArrayViewMut<'a, D> {
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut [D]> {
        self.container.get_mut(entity, self.cycle)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut [D]> {
        self.container.iter_mut()
    }
}

impl<'a, D: StaticDataType> StaticArrayView<D> for StaticArrayViewMut<'a, D> {
    fn get(&self, entity: Entity) -> Option<&[D]> {
        self.container.get(entity)
    }
}

impl<'a, D: StaticDataType> Index<Entity> for StaticArrayViewMut<'a, D> {
    type Output = [D];

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, D: StaticDataType> IndexMut<Entity> for StaticArrayViewMut<'a, D> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}

pub struct ArrayViewRef<'a> {
    pub(crate) container: Ref<'a, Box<dyn AnyArrayContainer>>,
}

pub struct ArrayViewMut<'a> {
    pub(crate) container: RefMut<'a, Box<dyn AnyArrayContainer>>,
    pub(crate) cycle: u32,
}
