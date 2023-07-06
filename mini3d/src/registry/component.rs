use std::collections::HashMap;

use crate::{
    ecs::{
        container::{AnyComponentContainer, ComponentContainer},
        dynamic::DynamicComponent,
        entity::Entity,
        error::ECSError,
        singleton::{AnyComponentSingleton, ComponentSingleton},
    },
    script::property::{Property, PropertyReflection},
    serialize::Serialize,
    uid::UID,
};

use super::error::RegistryError;

pub struct EntityResolver;

impl EntityResolver {
    pub fn resolve(&self, entity: Entity) -> Result<Entity, ECSError> {
        // TODO: Resolve entity
        Ok(entity)
    }
}

pub trait Component: Serialize + PropertyReflection + Default + 'static {
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
    fn create_container(&self) -> Box<dyn AnyComponentContainer>;
    fn create_singleton(&self) -> Box<dyn AnyComponentSingleton>;
    fn properties(&self) -> &[Property];
}

pub(crate) struct ComponentReflection<C: Component> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Component> AnyComponentReflection for ComponentReflection<C> {
    fn create_container(&self) -> Box<dyn AnyComponentContainer> {
        Box::new(ComponentContainer::<C>::new())
    }

    fn create_singleton(&self) -> Box<dyn AnyComponentSingleton> {
        Box::new(ComponentSingleton::<C>::new(C::default()))
    }

    fn properties(&self) -> &[Property] {
        C::properties()
    }
}

pub(crate) struct ComponentDefinition {
    pub(crate) name: String,
    pub(crate) reflection: Box<dyn AnyComponentReflection>,
    pub(crate) kind: ComponentKind,
}

#[derive(Default)]
pub(crate) struct ComponentRegistry {
    pub(crate) definitions: HashMap<UID, ComponentDefinition>,
}

impl ComponentRegistry {
    fn define(
        &mut self,
        name: &str,
        kind: ComponentKind,
        reflection: Box<dyn AnyComponentReflection>,
    ) -> Result<UID, RegistryError> {
        let uid: UID = name.into();
        if self.definitions.contains_key(&uid) {
            return Err(RegistryError::DuplicatedComponentDefinition {
                name: name.to_string(),
            });
        }
        self.definitions.insert(
            uid,
            ComponentDefinition {
                name: name.to_string(),
                kind,
                reflection,
            },
        );
        Ok(uid)
    }

    pub(crate) fn define_static<C: Component>(&mut self, name: &str) -> Result<UID, RegistryError> {
        let reflection = ComponentReflection::<C> {
            _phantom: std::marker::PhantomData,
        };
        self.define(name, ComponentKind::Static, Box::new(reflection))
    }

    pub(crate) fn define_dynamic(&mut self, name: &str) -> Result<UID, RegistryError> {
        let reflection = ComponentReflection::<DynamicComponent> {
            _phantom: std::marker::PhantomData,
        };
        self.define(name, ComponentKind::Dynamic, Box::new(reflection))
    }

    pub(crate) fn get(&self, uid: UID) -> Option<&ComponentDefinition> {
        self.definitions.get(&uid)
    }
}
