use std::any::{Any, TypeId};

use crate::{
    asset::{
        container::{AnyAssetContainer, StaticAssetContainer},
        handle::{AssetHandle, DynamicAsset, StaticAsset},
    },
    ecs::{
        component::{AnyComponentContainer, ComponentTable, StaticComponentContainer},
        entity::Entity,
        error::SceneError,
        view::{
            ComponentViewMut, ComponentViewRef, StaticComponentViewMut, StaticComponentViewRef,
        },
    },
    script::reflection::{Property, Reflect},
    serialize::Serialize,
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::UID,
    },
};

use super::error::RegistryError;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ComponentId(SlotId);

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

pub trait ComponentHandle {
    type ViewRef<'a>;
    type ViewMut<'a>;
    type AssetHandle: AssetHandle;
    fn new(uid: UID, id: ComponentId) -> Self;
    fn uid(&self) -> UID;
    fn id(&self) -> ComponentId;
    fn view_ref<'a>(&self, components: &'a ComponentTable)
        -> Result<Self::ViewRef<'a>, SceneError>;
    fn view_mut<'a>(
        &self,
        components: &'a ComponentTable,
        cycle: u32,
    ) -> Result<Self::ViewMut<'a>, SceneError>;
    fn check_type(reflection: &dyn AnyComponentReflection) -> bool;
}

pub struct StaticComponent<C: Component> {
    _marker: std::marker::PhantomData<C>,
    uid: UID,
    id: ComponentId,
}

impl<C: Component> Default for StaticComponent<C> {
    fn default() -> Self {
        Self {
            _marker: std::marker::PhantomData,
            uid: UID::null(),
            id: ComponentId::default(),
        }
    }
}

impl<C: Component> ComponentHandle for StaticComponent<C> {
    type ViewRef<'a> = StaticComponentViewRef<'a, C>;
    type ViewMut<'a> = StaticComponentViewMut<'a, C>;
    type AssetHandle = StaticAsset<C>;

    fn new(uid: UID, id: ComponentId) -> Self {
        Self {
            _marker: std::marker::PhantomData,
            uid,
            id,
        }
    }

    fn uid(&self) -> UID {
        self.uid
    }

    fn id(&self) -> ComponentId {
        self.id
    }

    fn view_ref<'a>(
        &self,
        components: &'a ComponentTable,
    ) -> Result<Self::ViewRef<'a>, SceneError> {
        Ok(StaticComponentViewRef {
            container: components
                .containers
                .get(self.id.into())
                .unwrap()
                .try_borrow()
                .map_err(|_| SceneError::ContainerBorrowMut)?
                .as_any()
                .downcast_ref::<StaticComponentContainer<C>>()
                .ok_or(SceneError::ComponentTypeMismatch)?,
        })
    }

    fn view_mut<'a>(
        &self,
        components: &'a ComponentTable,
        cycle: u32,
    ) -> Result<Self::ViewMut<'a>, SceneError> {
        Ok(StaticComponentViewMut {
            container: components
                .containers
                .get(self.id.into())
                .unwrap()
                .try_borrow_mut()
                .map_err(|_| SceneError::ContainerBorrowMut)?
                .as_any_mut()
                .downcast_mut::<StaticComponentContainer<C>>()
                .ok_or(SceneError::ComponentTypeMismatch)?,
            cycle,
        })
    }

    fn check_type(reflection: &dyn AnyComponentReflection) -> bool {
        reflection.type_id() == TypeId::of::<StaticComponentReflection<C>>()
    }
}

pub struct DynamicComponent {
    uid: UID,
    id: ComponentId,
}

impl ComponentHandle for DynamicComponent {
    type ViewRef<'a> = ComponentViewRef<'a>;
    type ViewMut<'a> = ComponentViewMut<'a>;
    type AssetHandle = DynamicAsset;

    fn new(uid: UID, id: ComponentId) -> Self {
        Self { uid, id }
    }

    fn uid(&self) -> UID {
        self.uid
    }

    fn id(&self) -> ComponentId {
        self.id
    }

    fn view_ref<'a>(
        &self,
        components: &'a ComponentTable,
    ) -> Result<Self::ViewRef<'a>, SceneError> {
        Ok(ComponentViewRef {
            container: components
                .containers
                .get(self.id.into())
                .unwrap()
                .try_borrow()
                .map_err(|_| SceneError::ContainerBorrowMut)?,
        })
    }

    fn view_mut<'a>(
        &self,
        components: &'a ComponentTable,
        cycle: u32,
    ) -> Result<Self::ViewMut<'a>, SceneError> {
        Ok(ComponentViewMut {
            container: components
                .containers
                .get(self.id.into())
                .unwrap()
                .try_borrow_mut()
                .map_err(|_| SceneError::ContainerBorrowMut)?,
            cycle,
        })
    }

    fn check_type(reflection: &dyn AnyComponentReflection) -> bool {
        true // Dynamic handle is valid for both static and dynamic components
    }
}

pub struct EntityResolver;

impl EntityResolver {
    pub fn resolve(&self, entity: Entity) -> Result<Entity, SceneError> {
        // TODO: Resolve entity
        Ok(entity)
    }
}
pub trait Component: Default + Serialize + Reflect + 'static {
    const NAME: &'static str;
    const UID: UID = UID::from(Self::NAME);
    fn resolve_entities(&mut self, resolver: &EntityResolver) -> Result<(), SceneError> {
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
}

pub(crate) struct StaticComponentReflection<C: Component> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Component> AnyComponentReflection for StaticComponentReflection<C> {
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
}

pub(crate) struct ComponentDefinition {
    pub(crate) name: String,
    pub(crate) reflection: Box<dyn AnyComponentReflection>,
    pub(crate) kind: ComponentKind,
}

#[derive(Default)]
pub(crate) struct ComponentRegistry {
    definitions: SlotMap<ComponentDefinition>,
}

impl ComponentRegistry {
    fn define(
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
        let id = self.definitions.add(ComponentDefinition {
            name: name.to_string(),
            kind,
            reflection,
        });
        Ok(())
    }

    pub(crate) fn define_static<C: Component>(&mut self, name: &str) -> Result<(), RegistryError> {
        let reflection = StaticComponentReflection::<C> {
            _phantom: std::marker::PhantomData,
        };
        self.define(name, ComponentKind::Static, Box::new(reflection))
    }

    pub(crate) fn define_dynamic(&mut self, name: &str) -> Result<ComponentId, RegistryError> {
        unimplemented!()
    }

    pub(crate) fn define_tag(&mut self, name: &str) -> Result<ComponentId, RegistryError> {
        unimplemented!()
    }

    pub(crate) fn definition<H: ComponentHandle>(
        &self,
        handle: H,
    ) -> Result<&ComponentDefinition, RegistryError> {
        self.definitions
            .get(handle.id().into())
            .ok_or(RegistryError::AssetDefinitionNotFound)
    }

    pub(crate) fn find_id(&self, component: UID) -> Option<ComponentId> {
        self.definitions
            .iter()
            .find(|(_, def)| UID::new(&def.name) == component)
            .map(|(id, _)| id.into())
    }

    pub fn find<H: ComponentHandle>(&mut self, component: UID) -> Option<H> {
        if let Some(id) = self.find_id(component) {
            if !H::check_type(self.definitions[id.into()].reflection.as_ref()) {
                return None;
            } else {
                return Some(H::new(component, id));
            }
        } else {
            None
        }
    }
}
