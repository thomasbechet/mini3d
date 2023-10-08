use std::{
    any::TypeId,
    cell::{Ref, RefMut},
};

use crate::{
    ecs::{
        container::single::NativeSingleContainer,
        entity::Entity,
        error::ECSError,
        view::single::{NativeSingleViewMut, NativeSingleViewRef},
    },
    utils::slotmap::SlotId,
};

use super::component::{
    Component, ComponentTypeTrait, PrivateComponentTableMut, PrivateComponentTableRef,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentType(pub(crate) SlotId);

impl ComponentTypeTrait for ComponentType {
    type SingleViewRef<'a> = SingleViewRef<'a>;
    type SingleViewMut<'a> = SingleViewMut<'a>;
    type Data = ();

    fn new(id: ComponentType) -> Self {
        id
    }

    fn id(&self) -> ComponentType {
        *self
    }

    fn single_view_ref<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
    ) -> Result<Self::SingleViewRef<'a>, ECSError> {
        Ok(SingleViewRef {
            container: components
                .0
                .containers
                .get(self.0)
                .unwrap()
                .try_borrow()
                .map_err(|_| ECSError::ContainerBorrowMut)?,
        })
    }

    fn single_view_mut<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
        cycle: u32,
    ) -> Result<Self::SingleViewMut<'a>, ECSError> {
        Ok(SingleViewMut {
            container: components
                .0
                .containers
                .get(self.0)
                .unwrap()
                .try_borrow_mut()
                .map_err(|_| ECSError::ContainerBorrowMut)?,
            cycle,
        })
    }

    fn check_type_id(_id: TypeId) -> bool {
        true // Dynamic handle is valid for both static and dynamic components
    }

    fn insert_single_container(
        &self,
        components: PrivateComponentTableMut,
        entity: Entity,
        data: Self::Data,
        cycle: u32,
    ) {
    }
}

impl<C: Component> ComponentTypeTrait for NativeComponentType<C> {
    type SingleViewRef<'a> = NativeSingleViewRef<'a, C>;
    type SingleViewMut<'a> = NativeSingleViewMut<'a, C>;
    type Data = C;

    fn new(id: NativeComponentType) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id,
        }
    }

    fn id(&self) -> NativeComponentType {
        self.id
    }

    fn single_view_ref<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
    ) -> Result<Self::SingleViewRef<'a>, ECSError> {
        Ok(NativeSingleViewRef {
            container: Ref::map(
                components
                    .0
                    .containers
                    .get(self.id.0)
                    .unwrap()
                    .try_borrow()
                    .map_err(|_| ECSError::ContainerBorrowMut)?,
                |r| {
                    r.as_any()
                        .downcast_ref::<NativeSingleContainer<C>>()
                        .unwrap()
                },
            ),
        })
    }

    fn single_view_mut<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
        cycle: u32,
    ) -> Result<Self::SingleViewMut<'a>, ECSError> {
        Ok(NativeSingleViewMut {
            container: RefMut::map(
                components
                    .0
                    .containers
                    .get(self.id.0)
                    .unwrap()
                    .try_borrow_mut()
                    .map_err(|_| ECSError::ContainerBorrowMut)?,
                |r| {
                    r.as_any_mut()
                        .downcast_mut::<NativeSingleContainer<C>>()
                        .unwrap()
                },
            ),
            cycle,
        })
    }

    fn check_type_id(id: TypeId) -> bool {
        id == TypeId::of::<C>()
    }

    fn insert_single_container(
        &self,
        components: PrivateComponentTableMut,
        entity: Entity,
        data: Self::Data,
        cycle: u32,
    ) {
        components
            .0
            .containers
            .get_mut(self.id.0)
            .expect("Component container not found while adding entity")
            .get_mut()
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<C>>()
            .expect("Component type mismatch while adding static component")
            .add(entity, data, cycle);
    }
}
