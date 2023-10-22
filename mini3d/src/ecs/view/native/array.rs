use std::{
    cell::{Ref, RefMut},
    ops::{Deref, Index, IndexMut},
};

use crate::{
    ecs::{
        container::native::array::NativeArrayContainer,
        entity::Entity,
        view::{ComponentViewMut, ComponentViewRef},
    },
    feature::core::component::{
        Component, ComponentId, PrivateComponentTableMut, PrivateComponentTableRef,
    },
};

pub trait NativeArrayView<C: Component> {
    fn get(&self, entity: Entity) -> Option<&[C]>;
}

// Native array reference

pub struct NativeArrayViewRef<'a, C: Component> {
    pub(crate) container: Ref<'a, NativeArrayContainer<C>>,
}

impl<'a, C: Component> NativeArrayViewRef<'a, C> {
    pub fn iter(&self) -> impl Iterator<Item = &[C]> {
        self.container.iter()
    }
}

impl<'a, C: Component> NativeArrayView<C> for NativeArrayViewRef<'a, C> {
    fn get(&self, entity: Entity) -> Option<&[C]> {
        self.container.get(entity)
    }
}

impl<'a, C: Component> Index<Entity> for NativeArrayViewRef<'a, C> {
    type Output = [C];

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> ComponentViewRef for NativeArrayViewRef<'a, C> {
    fn view(table: PrivateComponentTableRef, id: ComponentId) -> Self {
        Self {
            container: Ref::map(
                table.0.containers.get(id.0).unwrap().try_borrow().unwrap(),
                |r| {
                    r.as_any()
                        .downcast_ref::<NativeArrayContainer<C>>()
                        .unwrap()
                },
            ),
        }
    }
}

// Native array mutable reference

pub struct NativeArrayViewMut<'a, C: Component> {
    pub(crate) container: RefMut<'a, NativeArrayContainer<C>>,
    pub(crate) cycle: u32,
}

impl<'a, C: Component> NativeArrayViewMut<'a, C> {
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut [C]> {
        self.container.get_mut(entity, self.cycle)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut [C]> {
        self.container.iter_mut()
    }
}

impl<'a, C: Component> NativeArrayView<C> for NativeArrayViewMut<'a, C> {
    fn get(&self, entity: Entity) -> Option<&[C]> {
        self.container.get(entity)
    }
}

impl<'a, C: Component> Index<Entity> for NativeArrayViewMut<'a, C> {
    type Output = [C];

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> IndexMut<Entity> for NativeArrayViewMut<'a, C> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> ComponentViewMut for NativeArrayViewMut<'a, C> {
    fn view_mut(table: PrivateComponentTableMut, id: ComponentId, cycle: u32) -> Self {
        Self {
            container: RefMut::map(
                table
                    .0
                    .containers
                    .get_mut(id.0)
                    .unwrap()
                    .try_borrow()
                    .unwrap(),
                |r| {
                    r.as_any_mut()
                        .downcast_mut::<NativeArrayContainer<C>>()
                        .unwrap()
                },
            ),
            cycle,
        }
    }
}

// Native array singleton reference

pub struct NativeArraySingletonRef<'a, C: Component>(Ref<'a, NativeArrayContainer<C>>);

impl<'a, C: Component> Deref for NativeArraySingletonRef<'a, C> {
    type Target = [C];

    fn deref(&self) -> &Self::Target {
        self.0.iter().next().unwrap()
    }
}
