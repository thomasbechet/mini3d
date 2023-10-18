use std::{
    cell::{Ref, RefMut},
    ops::{Deref, DerefMut, Index, IndexMut},
};

use crate::{
    ecs::{
        container::native::single::NativeSingleContainer,
        entity::Entity,
        view::{ComponentViewMut, ComponentViewRef},
    },
    feature::core::component_type::{
        Component, ComponentId, PrivateComponentTableMut, PrivateComponentTableRef,
    },
};

pub trait NativeSingleView<C: Component> {
    fn get(&self, entity: Entity) -> Option<&C>;
}

// Native single reference

pub struct NativeSingleViewRef<'a, C: Component> {
    pub(crate) container: Ref<'a, NativeSingleContainer<C>>,
}

impl<'a, C: Component> NativeSingleViewRef<'a, C> {
    pub fn iter(&self) -> impl Iterator<Item = &C> {
        self.container.iter()
    }

    pub fn singleton(self) -> Option<NativeSingleSingletonRef<'a, C>> {
        // TODO: Ensure has at lease one entity
        Some(NativeSingleSingletonRef(self.container))
    }
}

impl<'a, C: Component> NativeSingleView<C> for NativeSingleViewRef<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.container.get(entity)
    }
}

impl<'a, C: Component> Index<Entity> for NativeSingleViewRef<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> ComponentViewRef for NativeSingleViewRef<'a, C> {
    fn view(table: PrivateComponentTableRef, id: ComponentId) -> Self {
        Self {
            container: Ref::map(
                table.0.containers.get(id.0).unwrap().try_borrow().unwrap(),
                |r| {
                    r.as_any()
                        .downcast_ref::<NativeSingleContainer<C>>()
                        .unwrap()
                },
            ),
        }
    }
}

// Native single mutable reference

pub struct NativeSingleViewMut<'a, C: Component> {
    pub(crate) container: RefMut<'a, NativeSingleContainer<C>>,
    pub(crate) cycle: u32,
}

impl<'a, C: Component> NativeSingleViewMut<'a, C> {
    pub fn get_mut(&mut self, entity: Entity) -> Option<&mut C> {
        self.container.get_mut(entity, self.cycle)
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut C> {
        self.container.iter_mut()
    }

    pub fn singleton(self) -> Option<NativeSingleSingletonMut<'a, C>> {
        // TODO: Ensure has at lease one entity
        Some(NativeSingleSingletonMut {
            container: self.container,
            cycle: self.cycle,
        })
    }
}

impl<'a, C: Component> NativeSingleView<C> for NativeSingleViewMut<'a, C> {
    fn get(&self, entity: Entity) -> Option<&C> {
        self.container.get(entity)
    }
}

impl<'a, C: Component> Index<Entity> for NativeSingleViewMut<'a, C> {
    type Output = C;

    fn index(&self, entity: Entity) -> &Self::Output {
        self.get(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> IndexMut<Entity> for NativeSingleViewMut<'a, C> {
    fn index_mut(&mut self, entity: Entity) -> &mut Self::Output {
        self.get_mut(entity).expect("Entity not found")
    }
}

impl<'a, C: Component> ComponentViewMut for NativeSingleViewMut<'a, C> {
    fn view_mut(table: PrivateComponentTableMut, id: ComponentId, cycle: u32) -> Self {
        Self {
            container: RefMut::map(
                table
                    .0
                    .containers
                    .get_mut(id.0)
                    .unwrap()
                    .try_borrow_mut()
                    .unwrap(),
                |r| {
                    r.as_any_mut()
                        .downcast_mut::<NativeSingleContainer<C>>()
                        .unwrap()
                },
            ),
            cycle,
        }
    }
}

// Native single singleton reference

pub struct NativeSingleSingletonRef<'a, C: Component>(Ref<'a, NativeSingleContainer<C>>);

impl<'a, C: Component> Deref for NativeSingleSingletonRef<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.0.iter().next().unwrap()
    }
}

// Native singleton mutable reference

pub struct NativeSingleSingletonMut<'a, C: Component> {
    container: RefMut<'a, NativeSingleContainer<C>>,
    cycle: u32,
}

impl<'a, C: Component> Deref for NativeSingleSingletonMut<'a, C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        self.container.iter().next().unwrap()
    }
}

impl<'a, C: Component> DerefMut for NativeSingleSingletonMut<'a, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.container.iter_mut().next().unwrap()
    }
}
