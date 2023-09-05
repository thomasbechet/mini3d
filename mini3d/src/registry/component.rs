use std::{
    any::TypeId,
    cell::{Ref, RefMut},
};

use crate::{
    asset::{
        container::{AnyAssetContainer, StaticAssetContainer},
        handle::{Asset, AssetHandle, StaticAsset},
    },
    ecs::{
        component::{AnyComponentContainer, ComponentTable, StaticComponentContainer},
        entity::Entity,
        error::ECSError,
        view::{
            ComponentViewMut, ComponentViewRef, StaticComponentViewMut, StaticComponentViewRef,
        },
    },
    script::reflection::{Property, Reflect},
    serialize::Serialize,
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
        uid::UID,
    },
};

use super::error::RegistryError;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(SlotId);

impl From<SlotId> for ComponentId {
    fn from(id: SlotId) -> Self {
        Self(id)
    }
}

impl From<ComponentId> for SlotId {
    fn from(id: ComponentId) -> Self {
        id.0
    }
}

pub struct PrivateComponentTableRef<'a>(pub(crate) &'a ComponentTable);
pub struct PrivateComponentTableMut<'a>(pub(crate) &'a mut ComponentTable);

pub trait ComponentHandle: Copy {
    type ViewRef<'a>;
    type ViewMut<'a>;
    type AssetHandle: AssetHandle;
    type Data: Default;
    fn new(id: ComponentId) -> Self;
    fn id(&self) -> ComponentId;
    fn view_ref<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
    ) -> Result<Self::ViewRef<'a>, ECSError>;
    fn view_mut<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
        cycle: u32,
    ) -> Result<Self::ViewMut<'a>, ECSError>;
    fn check_type_id(id: TypeId) -> bool;
    fn insert_container(
        &self,
        components: PrivateComponentTableMut,
        entity: Entity,
        data: Self::Data,
        cycle: u32,
    );
}

pub struct StaticComponent<C: ComponentData> {
    _marker: std::marker::PhantomData<C>,
    id: ComponentId,
}

impl<C: ComponentData> Clone for StaticComponent<C> {
    fn clone(&self) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: self.id,
        }
    }
}

impl<C: ComponentData> Copy for StaticComponent<C> {}

impl<C: ComponentData> Default for StaticComponent<C> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id: ComponentId::default(),
        }
    }
}

impl<C: ComponentData> ComponentHandle for StaticComponent<C> {
    type ViewRef<'a> = StaticComponentViewRef<'a, C>;
    type ViewMut<'a> = StaticComponentViewMut<'a, C>;
    type AssetHandle = StaticAsset<C>;
    type Data = C;

    fn new(id: ComponentId) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            id,
        }
    }

    fn id(&self) -> ComponentId {
        self.id
    }

    fn view_ref<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
    ) -> Result<Self::ViewRef<'a>, ECSError> {
        Ok(StaticComponentViewRef {
            container: Ref::map(
                components
                    .0
                    .containers
                    .get(self.id.into())
                    .unwrap()
                    .try_borrow()
                    .map_err(|_| ECSError::ContainerBorrowMut)?,
                |r| {
                    r.as_any()
                        .downcast_ref::<StaticComponentContainer<C>>()
                        .unwrap()
                },
            ),
        })
    }

    fn view_mut<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
        cycle: u32,
    ) -> Result<Self::ViewMut<'a>, ECSError> {
        Ok(StaticComponentViewMut {
            container: RefMut::map(
                components
                    .0
                    .containers
                    .get(self.id.into())
                    .unwrap()
                    .try_borrow_mut()
                    .map_err(|_| ECSError::ContainerBorrowMut)?,
                |r| {
                    r.as_any_mut()
                        .downcast_mut::<StaticComponentContainer<C>>()
                        .unwrap()
                },
            ),
            cycle,
        })
    }

    fn check_type_id(id: TypeId) -> bool {
        id == TypeId::of::<C>()
    }

    fn insert_container(
        &self,
        components: PrivateComponentTableMut,
        entity: Entity,
        data: Self::Data,
        cycle: u32,
    ) {
        components
            .0
            .containers
            .get_mut(self.id.into())
            .expect("Component container not found while adding entity")
            .get_mut()
            .as_any_mut()
            .downcast_mut::<StaticComponentContainer<C>>()
            .expect("Component type mismatch while adding static component")
            .add(entity, data, cycle);
    }
}

#[derive(Clone, Copy)]
pub struct Component {
    id: ComponentId,
}

impl ComponentHandle for Component {
    type ViewRef<'a> = ComponentViewRef<'a>;
    type ViewMut<'a> = ComponentViewMut<'a>;
    type AssetHandle = Asset;
    type Data = ();

    fn new(id: ComponentId) -> Self {
        Self { id }
    }

    fn id(&self) -> ComponentId {
        self.id
    }

    fn view_ref<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
    ) -> Result<Self::ViewRef<'a>, ECSError> {
        Ok(ComponentViewRef {
            container: components
                .0
                .containers
                .get(self.id.into())
                .unwrap()
                .try_borrow()
                .map_err(|_| ECSError::ContainerBorrowMut)?,
        })
    }

    fn view_mut<'a>(
        &self,
        components: PrivateComponentTableRef<'a>,
        cycle: u32,
    ) -> Result<Self::ViewMut<'a>, ECSError> {
        Ok(ComponentViewMut {
            container: components
                .0
                .containers
                .get(self.id.into())
                .unwrap()
                .try_borrow_mut()
                .map_err(|_| ECSError::ContainerBorrowMut)?,
            cycle,
        })
    }

    fn check_type_id(_id: TypeId) -> bool {
        true // Dynamic handle is valid for both static and dynamic components
    }

    fn insert_container(
        &self,
        components: PrivateComponentTableMut,
        entity: Entity,
        data: Self::Data,
        cycle: u32,
    ) {
    }
}

pub struct EntityResolver;

impl EntityResolver {
    pub fn resolve(&self, entity: Entity) -> Result<Entity, ECSError> {
        // TODO: Resolve entity
        Ok(entity)
    }
}
pub trait ComponentData: Default + Serialize + Reflect + 'static {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn resolve_entities(&mut self, resolver: &EntityResolver) -> Result<(), ECSError> {
        let _ = resolver;
        Ok(())
    }
}

pub(crate) enum ComponentKind {
    Static,
    Dynamic,
    Tag,
}

pub(crate) trait AnyComponentReflection {
    fn create_asset_container(&self) -> Box<dyn AnyAssetContainer>;
    fn create_scene_container(&self) -> Box<dyn AnyComponentContainer>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
    fn type_id(&self) -> TypeId;
}

pub(crate) struct StaticComponentReflection<C: ComponentData> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: ComponentData> AnyComponentReflection for StaticComponentReflection<C> {
    fn create_asset_container(&self) -> Box<dyn AnyAssetContainer> {
        Box::<StaticAssetContainer<C>>::default()
    }

    fn create_scene_container(&self) -> Box<dyn AnyComponentContainer> {
        Box::new(StaticComponentContainer::<C>::with_capacity(128))
    }

    fn find_property(&self, name: &str) -> Option<&Property> {
        C::PROPERTIES.iter().find(|p| p.name == name)
    }

    fn properties(&self) -> &[Property] {
        C::PROPERTIES
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<C>()
    }
}

pub(crate) const MAX_COMPONENT_NAME_LEN: usize = 64;

pub(crate) struct ComponentEntry {
    pub(crate) name: AsciiArray<MAX_COMPONENT_NAME_LEN>,
    pub(crate) reflection: Box<dyn AnyComponentReflection>,
    pub(crate) kind: ComponentKind,
}

#[derive(Default)]
pub(crate) struct ComponentRegistry {
    entries: SlotMap<ComponentEntry>,
}

impl ComponentRegistry {
    fn add(
        &mut self,
        name: &str,
        kind: ComponentKind,
        reflection: Box<dyn AnyComponentReflection>,
    ) -> Result<(), RegistryError> {
        let uid: UID = name.into();
        if self.find_id(uid).is_some() {
            return Err(RegistryError::DuplicatedComponentDefinition {
                name: name.to_string(),
            });
        }
        let id = self.entries.add(ComponentEntry {
            name: name.into(),
            kind,
            reflection,
        });
        Ok(())
    }

    pub(crate) fn add_static<C: ComponentData>(&mut self, name: &str) -> Result<(), RegistryError> {
        let reflection = StaticComponentReflection::<C> {
            _phantom: std::marker::PhantomData,
        };
        self.add(name, ComponentKind::Static, Box::new(reflection))
    }

    pub(crate) fn add_dynamic(&mut self, name: &str) -> Result<ComponentId, RegistryError> {
        unimplemented!()
    }

    pub(crate) fn add_tag(&mut self, name: &str) -> Result<ComponentId, RegistryError> {
        unimplemented!()
    }

    pub(crate) fn definition<H: ComponentHandle>(
        &self,
        handle: H,
    ) -> Result<&ComponentEntry, RegistryError> {
        self.entries
            .get(handle.id().into())
            .ok_or(RegistryError::AssetDefinitionNotFound)
    }

    pub(crate) fn find_id(&self, component: UID) -> Option<ComponentId> {
        self.entries
            .iter()
            .find(|(_, def)| UID::new(&def.name) == component)
            .map(|(id, _)| id.into())
    }

    pub fn find<H: ComponentHandle>(&self, component: UID) -> Option<H> {
        if let Some(id) = self.find_id(component) {
            if !H::check_type_id(self.entries[id.into()].reflection.type_id()) {
                None
            } else {
                Some(H::new(id))
            }
        } else {
            None
        }
    }
}
