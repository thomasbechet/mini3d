use std::collections::HashMap;

use serde::{Serialize, Deserialize, Serializer};

use crate::{uid::UID, ecs::{container::{AnyComponentContainer, ComponentContainer}, singleton::{AnySingleton, Singleton}, entity::Entity, dynamic::DynamicComponent, error::ECSError}};

use super::error::RegistryError;

pub struct EntityResolver;

impl EntityResolver {
    pub fn resolve(&self, entity: Entity) -> Result<Entity, ECSError> {
        // TODO: Resolve entity
        Ok(entity)
    }
}

pub trait Component: Serialize + for<'de> Deserialize<'de> + 'static {
    fn resolve_entities(&mut self, resolver: &EntityResolver) -> Result<(), ECSError> { Ok(()) }
}

pub trait ComponentInspector {
    type C: Component;
    fn write_property(&self, component: &mut Self::C, property: UID, value: &serde_json::Value) -> Result<(), ECSError>;
    fn read_property(&self, component: &Self::C, property: UID) -> Option<serde_json::Value>;
}

pub(crate) enum ComponentKind {
    Static,
    Dynamic,
}

pub(crate) trait AnyComponentReflection {
    fn create_container(&self) -> Box<dyn AnyComponentContainer>;
    fn serialize_container<'a>(&'a self, container: &'a dyn AnyComponentContainer) -> Box<dyn erased_serde::Serialize + 'a>;
    fn deserialize_container(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnyComponentContainer>, erased_serde::Error>;
    fn serialize_singleton<'a>(&'a self, singleton: &'a dyn AnySingleton) -> Box<dyn erased_serde::Serialize + 'a>;
    fn deserialize_singleton(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnySingleton>, erased_serde::Error>;
}

pub(crate) struct ComponentReflection<C: Component> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Component> AnyComponentReflection for ComponentReflection<C> {
    
    fn create_container(&self) -> Box<dyn AnyComponentContainer> {
        Box::new(ComponentContainer::<C>::new())
    }

    fn serialize_container<'a>(&'a self, container: &'a dyn AnyComponentContainer) -> Box<dyn erased_serde::Serialize + 'a> {
        struct SerializeContext<'a, C: Component> {
            container: &'a ComponentContainer<C>,
        }
        impl<'a, C: Component> Serialize for SerializeContext<'a, C> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                ComponentContainer::<C>::serialize(self.container, serializer)
            }
        }
        Box::new(SerializeContext { container: container.as_any().downcast_ref::<ComponentContainer<C>>().expect("Invalid container type") })
    }

    fn deserialize_container(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnyComponentContainer>, erased_serde::Error> {
        Ok(Box::new(ComponentContainer::<C>::deserialize(deserializer)?))
    }

    fn serialize_singleton<'a>(&'a self, singleton: &'a dyn AnySingleton) -> Box<dyn erased_serde::Serialize + 'a> {
        struct SerializeContext<'a, C: Component> {
            singleton: &'a Singleton<C>,
        }
        impl<'a, C: Component> Serialize for SerializeContext<'a, C> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.singleton.component.borrow().serialize(serializer)
            }
        }
        Box::new(SerializeContext { singleton: singleton.as_any().downcast_ref::<Singleton<C>>().expect("Invalid singleton type") })
    }

    fn deserialize_singleton(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnySingleton>, erased_serde::Error> {
        Ok(Box::new(Singleton::<C>::new(C::deserialize(deserializer)?)))
    }
}

pub struct ComponentProperty {
    pub(crate) name: String,
    pub(crate) format: UID,
    pub(crate) editable: bool,
}

impl ComponentProperty {

    pub const BOOL: UID = UID::new("bool");
    pub const FLOAT: UID = UID::new("float");
    pub const VEC2: UID = UID::new("vec2");
    pub const VEC3: UID = UID::new("vec3");
    pub const INPUT_ACTION: UID = UID::new("input_action");
    pub const INPUT_AXIS: UID = UID::new("input_axis");

    pub fn new(name: &str, format: UID) -> Self {
        Self { name: name.to_string(), format, editable: true }
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
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

    fn define(&mut self, name: &str, kind: ComponentKind, reflection: Box<dyn AnyComponentReflection>) -> Result<UID, RegistryError> {
        let uid: UID = name.into();
        if self.definitions.contains_key(&uid) {
            return Err(RegistryError::DuplicatedComponentDefinition { name: name.to_string() });
        }
        self.definitions.insert(uid, ComponentDefinition { name: name.to_string(), kind, reflection });
        Ok(uid)
    }

    pub(crate) fn define_static<C: Component>(&mut self, name: &str) -> Result<UID, RegistryError> {
        let reflection = ComponentReflection::<C> { _phantom: std::marker::PhantomData };
        Ok(self.define(name, ComponentKind::Static, Box::new(reflection))?)
    }

    pub(crate) fn define_dynamic(&mut self, name: &str) -> Result<UID, RegistryError> {
        let reflection = ComponentReflection::<DynamicComponent> { _phantom: std::marker::PhantomData };
        Ok(self.define(name, ComponentKind::Dynamic, Box::new(reflection))?)
    }

    pub(crate) fn get(&self, uid: UID) -> Option<&ComponentDefinition> {
        self.definitions.get(&uid)
    }
}