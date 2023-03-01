use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize, Serializer};

use crate::{uid::UID, feature::asset::runtime_component::FieldType, ecs::{container::{AnyComponentContainer, ComponentContainer, DynamicComponent1, DynamicComponent2, DynamicComponent3, DynamicComponent4, DynamicComponent5}, component::Component, singleton::{AnySingleton, Singleton}}};

#[derive(Clone, Serialize, Deserialize)]
pub struct DynamicComponentDefinition {
    pub fields: HashMap<String, FieldType>,
}

pub(crate) enum ComponentKind {
    Static,
    Dynamic(DynamicComponentDefinition),
}

pub(crate) struct AnyComponentContainerDeserializeSeed;

pub(crate) trait AnyComponentDefinitionReflection {
    fn create_container(&self) -> Box<dyn AnyComponentContainer>;
    fn serialize_container<'a>(&'a self, container: &'a dyn AnyComponentContainer) -> Box<dyn erased_serde::Serialize + 'a>;
    fn deserialize_container(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnyComponentContainer>>;
    fn serialize_singleton<'a>(&'a self, singleton: &'a dyn AnySingleton) -> Box<dyn erased_serde::Serialize + 'a>;
    fn deserialize_singleton(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnySingleton>>;
}

pub(crate) struct ComponentDefinitionReflection<C: Component> {
    _phantom: std::marker::PhantomData<C>,
}

impl<C: Component> AnyComponentDefinitionReflection for ComponentDefinitionReflection<C> {
    
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

    fn deserialize_container(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnyComponentContainer>> {
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

    fn deserialize_singleton(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnySingleton>> {
        Ok(Box::new(Singleton::<C>::new(C::deserialize(deserializer)?)))
    }
}

pub(crate) struct ComponentDefinition {
    pub(crate) name: String,
    pub(crate) kind: ComponentKind,
    pub(crate) reflection: Box<dyn AnyComponentDefinitionReflection>,
}

#[derive(Default)]
pub(crate) struct ComponentRegistry {
    components: HashMap<UID, ComponentDefinition>,
}

impl ComponentRegistry {

    fn define(&mut self, name: &str, kind: ComponentKind, reflection: Box<dyn AnyComponentDefinitionReflection>) -> Result<UID> {
        let uid: UID = name.into();
        if self.components.contains_key(&uid) {
            return Err(anyhow!("Component with name '{}' already defined", name));
        }
        self.components.insert(uid, ComponentDefinition { name: name.to_string(), kind, reflection });
        Ok(uid)
    }

    pub(crate) fn define_static<C: Component>(&mut self, name: &str) -> Result<UID> {
        let reflection = ComponentDefinitionReflection::<C> { _phantom: std::marker::PhantomData };
        let uid = self.define(name, ComponentKind::Static, Box::new(reflection))?;
        Ok(uid)
    }

    pub(crate) fn define_dynamic(&mut self, name: &str, definition: DynamicComponentDefinition) -> Result<UID> {
        let reflection: Box<dyn AnyComponentDefinitionReflection> = match definition.fields.len() {
            1 => Box::new(ComponentDefinitionReflection::<DynamicComponent1> { _phantom: std::marker::PhantomData }),
            2 => Box::new(ComponentDefinitionReflection::<DynamicComponent2> { _phantom: std::marker::PhantomData }),
            3 => Box::new(ComponentDefinitionReflection::<DynamicComponent3> { _phantom: std::marker::PhantomData }),
            4 => Box::new(ComponentDefinitionReflection::<DynamicComponent4> { _phantom: std::marker::PhantomData }),
            5 => Box::new(ComponentDefinitionReflection::<DynamicComponent5> { _phantom: std::marker::PhantomData }),
            _ => return Err(anyhow!("Runtime component with 0 or more than 5 fields not supported")),
        };
        let uid = self.define(name, ComponentKind::Dynamic(definition), reflection)?;
        Ok(uid)
    }

    pub(crate) fn get(&self, uid: UID) -> Option<&ComponentDefinition> {
        self.components.get(&uid)
    }
}