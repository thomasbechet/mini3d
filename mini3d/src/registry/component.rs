use crate::{
    asset::container::{AnyAssetContainer, StaticAssetContainer},
    ecs::{
        component::{AnyComponentContainer, ComponentHandle, StaticComponentContainer},
        entity::Entity,
        error::SceneError,
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
        self.definitions.get(handle.id())
    }

    pub fn find<H: ComponentHandle>(&mut self, component: UID) -> Result<H, RegistryError> {
        let id = self
            .registry
            .find_id(component)
            .ok_or(RegistryError::ComponentDefinitionNotFound { uid: component })?;
        self.components.preallocate(id, self.registry);
        Ok(H::new(component, id))
    }
}
