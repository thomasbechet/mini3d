use std::{
    cell::{Ref, RefMut},
    ops::{Deref, Index, IndexMut},
};

use crate::{
    ecs::{
        container::array::{AnyArrayContainer, StaticArrayContainer},
        entity::Entity,
    },
    registry::component::ComponentData,
};

pub trait StaticArrayView<C: ComponentData> {
    fn get(&self, entity: Entity) -> Option<&[C]>;
}

pub struct StaticArrayViewRef<'a, C: ComponentData> {
    pub(crate) container: Ref<'a, StaticArrayContainer<C>>,
}

impl<'a, C: ComponentData> StaticArrayViewRef<'a, C> {
    pub fn iter(&self) -> impl Iterator<Item = &C> {
        self.container.iter()
    }
}

impl<'a, C: ComponentData> StaticArrayView<C> for StaticArrayViewRef<'a, C> {
    fn get(&self, entity: Entity) -> Option<&[C]> {
        self.container.get(entity)
    }
}

impl<'a, C: ComponentData> Index<Entity> for StaticArrayViewRef<'a, C> {
    type Output = &'a [C];

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

pub struct StaticArrayArraytonRef<'a, C: ComponentData>(Ref<'a, StaticArrayContainer<C>>);

impl<'a, C: ComponentData> Deref for StaticArrayArraytonRef<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.0.iter().next().unwrap()
    }
}

pub struct StaticArrayViewMut<'a, C: ComponentData> {
    pub(crate) container: RefMut<'a, StaticArrayContainer<C>>,
    pub(crate) cycle: u32,
}

impl<'a, C: ComponentData> StaticArrayViewMut<'a, C> {
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.container.get_mut(entity, self.cycle)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut C> {
        self.container.iter_mut()
    }
}

impl<'a, C: ComponentData> StaticArrayView<C> for StaticArrayViewMut<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.container.get(entity)
    }
}

impl<'a, C: ComponentData> Index<Entity> for StaticArrayViewMut<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: ComponentData> IndexMut<Entity> for StaticArrayViewMut<'a, C> {
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
