use std::collections::HashMap;

use anyhow::{anyhow, Result};
use serde::{Serialize, Deserialize, Serializer};

use crate::{uid::UID, feature::asset::runtime_component::FieldType, ecs::{entity::Entity, container::{AnyComponentContainer, ComponentContainer, RuntimeComponent1, RuntimeComponent2, RuntimeComponent3, RuntimeComponent4, RuntimeComponent5}}};

pub struct EntityResolver;
pub struct ComponentContext;

pub trait Component: Serialize + for<'de> Deserialize<'de> + 'static {
    fn on_construct(&mut self, _entity: Entity, _ctx: &mut ComponentContext) -> Result<()> { Ok(()) }
    fn on_destruct(&mut self, _entity: Entity, _ctx: &mut ComponentContext) -> Result<()> { Ok(()) }
    fn resolve_entities(&mut self, _resolver: &EntityResolver) -> Result<()> { Ok(()) }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct RuntimeComponentDefinition {
    pub fields: HashMap<String, FieldType>,
}

pub(crate) enum ComponentKind {
    Compiled,
    Runtime(RuntimeComponentDefinition),
}

pub(crate) struct AnyComponentContainerDeserializeSeed;

pub(crate) trait AnyComponentDefinitionReflection {
    fn create_container(&self) -> Box<dyn AnyComponentContainer>;
    fn serialize_container<'a>(&'a self, container: &'a dyn AnyComponentContainer) -> Box<dyn erased_serde::Serialize + 'a>;
    fn deserialize_container(&self, deserializer: &mut dyn erased_serde::Deserializer) -> Result<Box<dyn AnyComponentContainer>>;
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

    pub(crate) fn define_compiled<C: Component>(&mut self, name: &str) -> Result<UID> {
        let reflection = ComponentDefinitionReflection::<C> { _phantom: std::marker::PhantomData };
        let uid = self.define(name, ComponentKind::Compiled, Box::new(reflection))?;
        Ok(uid)
    }

    pub(crate) fn define_runtime(&mut self, name: &str, definition: RuntimeComponentDefinition) -> Result<UID> {
        let reflection: Box<dyn AnyComponentDefinitionReflection> = match definition.fields.len() {
            1 => Box::new(ComponentDefinitionReflection::<RuntimeComponent1> { _phantom: std::marker::PhantomData }),
            2 => Box::new(ComponentDefinitionReflection::<RuntimeComponent2> { _phantom: std::marker::PhantomData }),
            3 => Box::new(ComponentDefinitionReflection::<RuntimeComponent3> { _phantom: std::marker::PhantomData }),
            4 => Box::new(ComponentDefinitionReflection::<RuntimeComponent4> { _phantom: std::marker::PhantomData }),
            5 => Box::new(ComponentDefinitionReflection::<RuntimeComponent5> { _phantom: std::marker::PhantomData }),
            _ => return Err(anyhow!("Runtime component with 0 or more than 5 fields not supported")),
        };
        let uid = self.define(name, ComponentKind::Runtime(definition), reflection)?;
        Ok(uid)
    }

    pub(crate) fn get(&self, uid: UID) -> Option<&ComponentDefinition> {
        self.components.get(&uid)
    }
}