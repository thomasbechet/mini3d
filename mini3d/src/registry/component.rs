use std::collections::HashMap;

use crate::{
    asset::container::{AnyAssetContainer, AssetContainer},
    ecs::{
        container::{AnySceneContainer, StaticSceneContainer},
        entity::Entity,
        error::ECSError,
        singleton::{AnySceneSingleton, StaticSceneSingleton},
    },
    script::reflection::{Property, Reflect},
    serialize::Serialize,
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::UID,
    },
};

use super::error::RegistryError;

pub(crate) type ComponentId = SlotId<ComponentDefinition>;

pub struct EntityResolver;

impl EntityResolver {
    pub fn resolve(&self, entity: Entity) -> Result<Entity, ECSError> {
        // TODO: Resolve entity
        Ok(entity)
    }
}
pub trait Component: Default + Serialize + Reflect + 'static {
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
}

pub(crate) trait AnyComponentReflection {
    fn create_asset_container(&self) -> Box<dyn AnyAssetContainer>;
    fn create_scene_container(&self) -> Box<dyn AnySceneContainer>;
    fn create_scene_singleton(&self) -> Box<dyn AnySceneSingleton>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
}

pub(crate) struct StaticComponentReflection<C: Component> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Component> AnyComponentReflection for StaticComponentReflection<C> {
    fn create_asset_container(&self) -> Box<dyn AnyAssetContainer> {
        Box::<AssetContainer<C>>::default()
    }

    fn create_scene_container(&self) -> Box<dyn AnySceneContainer> {
        Box::new(StaticSceneContainer::<C>::new())
    }

    fn create_scene_singleton(&self) -> Box<dyn AnySceneSingleton> {
        Box::new(StaticSceneSingleton::<C>::new(C::default()))
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
    lookup_cache: HashMap<UID, ComponentId>,
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
        });
        self.lookup_cache.insert(uid, id);
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

    pub(crate) fn find(&self, uid: UID) -> Option<(ComponentId, &ComponentDefinition)> {
        self.lookup_cache
            .get(&uid)
            .map(|id| (*id, self.definitions.get(*id).unwrap()))
    }

    pub(crate) fn get(&self, id: ComponentId) -> Option<&ComponentDefinition> {
        self.definitions.get(id)
    }
}
