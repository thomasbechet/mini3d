use std::{
    any::TypeId,
    cell::{Ref, RefMut},
};

use crate::{
    ecs::{
        container::{
            single::{AnySingleContainer, StaticSingleContainer},
            ContainerTable,
        },
        entity::Entity,
        error::ECSError,
        view::single::{SingleViewMut, SingleViewRef, StaticSingleViewMut, StaticSingleViewRef},
    },
    reflection::Property,
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
        uid::{ToUID, UID},
    },
};

use super::{datatype::StaticDataType, error::RegistryError};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentType(pub(crate) SlotId);

pub struct PrivateComponentTableRef<'a>(pub(crate) &'a ContainerTable);
pub struct PrivateComponentTableMut<'a>(pub(crate) &'a mut ContainerTable);

pub trait ComponentTypeTrait: Copy {
    type SingleViewRef<'a>;
    type SingleViewMut<'a>;
    // type ArrayViewRef<'a>;
    // type ArrayViewMut<'a>;
    type Data: Default;
    fn new(id: ComponentType) -> Self;
    fn id(&self) -> ComponentType;
    fn single_view_ref<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
    ) -> Result<Self::SingleViewRef<'a>, ECSError>;
    fn single_view_mut<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
        cycle: u32,
    ) -> Result<Self::SingleViewMut<'a>, ECSError>;
    fn check_type_id(id: TypeId) -> bool;
    fn insert_single_container(
        &self,
        components: PrivateComponentTableMut,
        entity: Entity,
        data: Self::Data,
        cycle: u32,
    );
}

pub struct StaticComponentType<D: StaticDataType> {
    _marker: std::marker::PhantomData<D>,
    pub(crate) id: ComponentType,
}

impl<D: StaticDataType> Clone for StaticComponentType<D> {
    fn clone(&self) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: self.id,
        }
    }
}

impl<D: StaticDataType> Copy for StaticComponentType<D> {}

impl<D: StaticDataType> Default for StaticComponentType<D> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: ComponentType::default(),
        }
    }
}

impl<D: StaticDataType> ComponentTypeTrait for StaticComponentType<D> {
    type SingleViewRef<'a> = StaticSingleViewRef<'a, D>;
    type SingleViewMut<'a> = StaticSingleViewMut<'a, D>;
    type Data = D;

    fn new(id: ComponentType) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id,
        }
    }

    fn id(&self) -> ComponentType {
        self.id
    }

    fn single_view_ref<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
    ) -> Result<Self::SingleViewRef<'a>, ECSError> {
        Ok(StaticSingleViewRef {
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
                        .downcast_ref::<StaticSingleContainer<D>>()
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
        Ok(StaticSingleViewMut {
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
                        .downcast_mut::<StaticSingleContainer<D>>()
                        .unwrap()
                },
            ),
            cycle,
        })
    }

    fn check_type_id(id: TypeId) -> bool {
        id == TypeId::of::<D>()
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
            .downcast_mut::<StaticSingleContainer<D>>()
            .expect("Component type mismatch while adding static component")
            .add(entity, data, cycle);
    }
}

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

pub(crate) enum ComponentKind {
    Static,
    Dynamic,
    Tag,
}

pub enum ComponentStorage {
    Single,
    Array(usize),
    List,
    Map,
}

pub(crate) trait AnyComponentReflection {
    fn create_scene_container(&self) -> Box<dyn AnySingleContainer>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
    fn type_id(&self) -> TypeId;
}

pub(crate) struct StaticComponentReflection<D: StaticDataType> {
    _phantom: std::marker::PhantomData<D>,
}

impl<D: StaticDataType> AnyComponentReflection for StaticComponentReflection<D> {
    fn create_scene_container(&self) -> Box<dyn AnySingleContainer> {
        Box::new(StaticSingleContainer::<D>::with_capacity(128))
    }

    fn find_property(&self, name: &str) -> Option<&Property> {
        D::PROPERTIES.iter().find(|p| p.name == name)
    }

    fn properties(&self) -> &[Property] {
        D::PROPERTIES
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<D>()
    }
}

pub(crate) const MAX_COMPONENT_NAME_LEN: usize = 64;

pub(crate) struct ComponentEntry {
    pub(crate) name: AsciiArray<MAX_COMPONENT_NAME_LEN>,
    pub(crate) reflection: Box<dyn AnyComponentReflection>,
    pub(crate) kind: ComponentKind,
    pub(crate) storage: ComponentStorage,
}

#[derive(Default)]
pub struct ComponentRegistry {
    pub(crate) entries: SlotMap<ComponentEntry>,
    pub(crate) changed: bool,
}

impl ComponentRegistry {
    fn add(
        &mut self,
        name: &str,
        storage: ComponentStorage,
        kind: ComponentKind,
        reflection: Box<dyn AnyComponentReflection>,
    ) -> Result<SlotId, RegistryError> {
        let uid: UID = name.into();
        if self.contains(uid) {
            return Err(RegistryError::DuplicatedComponent);
        }
        self.changed = true;
        Ok(self.entries.add(ComponentEntry {
            name: name.into(),
            kind,
            storage,
            reflection,
        }))
    }

    pub fn add_static<D: StaticDataType>(
        &mut self,
        name: &str,
        storage: ComponentStorage,
    ) -> Result<StaticComponentType<D>, RegistryError> {
        let reflection = StaticComponentReflection::<D> {
            _phantom: std::marker::PhantomData,
        };
        let id = self.add(name, storage, ComponentKind::Static, Box::new(reflection))?;
        Ok(StaticComponentType {
            _marker: std::marker::PhantomData,
            id: ComponentType(id),
        })
    }

    pub fn add_dynamic(
        &mut self,
        name: &str,
        storage: ComponentStorage,
    ) -> Result<ComponentType, RegistryError> {
        unimplemented!()
    }

    pub fn add_tag(&mut self, name: &str) -> Result<ComponentType, RegistryError> {
        unimplemented!()
    }

    pub(crate) fn definition<H: ComponentTypeTrait>(
        &self,
        handle: H,
    ) -> Result<&ComponentEntry, RegistryError> {
        self.entries
            .get(handle.id().0)
            .ok_or(RegistryError::ComponentNotFound)
    }

    pub fn find<H: ComponentTypeTrait>(&self, component: impl ToUID) -> Option<H> {
        // Find entry
        let component = component.to_uid();
        let component = self
            .entries
            .iter()
            .find(|(_, def)| UID::new(&def.name) == component)
            .map(|(id, _)| ComponentType(id));
        // Check type
        if let Some(id) = component {
            if !H::check_type_id(self.entries[id.0].reflection.type_id()) {
                None
            } else {
                Some(H::new(id))
            }
        } else {
            None
        }
    }

    pub fn contains(&self, component: impl ToUID) -> bool {
        let component = component.to_uid();
        self.find::<ComponentType>(component).is_some()
    }
}
