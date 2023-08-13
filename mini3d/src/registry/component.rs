use crate::{
    asset::container::{AnyAssetContainer, StaticAssetContainer},
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
}

pub struct DynamicComponent {
    uid: UID,
    id: ComponentId,
}

impl ComponentHandle for DynamicComponent {
    type ViewRef<'a> = ComponentViewRef<'a>;
    type ViewMut<'a> = ComponentViewMut<'a>;

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
    pub(crate) require_finalizer: bool,
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
        if self.find(uid).is_some() {
            return Err(RegistryError::DuplicatedComponentDefinition {
                name: name.to_string(),
            });
        }
        let id = self.definitions.add(ComponentDefinition {
            name: name.to_string(),
            kind,
            reflection,
            require_finalizer: true,
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

    pub(crate) fn definition<H: ComponentHandle>(&self, handle: H) -> Option<&ComponentDefinition> {
        self.definitions.get(handle.id().into())
    }

    pub fn find<H: ComponentHandle>(&mut self, component: UID) -> Result<Option<H>, RegistryError> {
        let id = self
            .find(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound { uid: component })?;
        Ok(H::new(component, id))
    }
}
