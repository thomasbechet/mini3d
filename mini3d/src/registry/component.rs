use std::collections::HashMap;

use crate::{uid::UID, ecs::{container::{ComponentContainer, AnyComponentContainer}, singleton::{AnyComponentSingleton, ComponentSingleton}, entity::Entity, dynamic::DynamicComponent, error::ECSError}, serialize::Serialize};

use super::error::RegistryError;

pub struct EntityResolver;

impl EntityResolver {
    pub fn resolve(&self, entity: Entity) -> Result<Entity, ECSError> {
        // TODO: Resolve entity
        Ok(entity)
    }
}

pub trait Component: Serialize + Default + 'static {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn resolve_entities(&mut self, resolver: &EntityResolver) -> Result<(), ECSError> { Ok(()) }
}

pub trait ComponentInspector {
    type C: Component;
    // fn write_property(&self, component: &mut Self::C, property: UID, value: &serde_json::Value) -> Result<(), ECSError>;
    // fn read_property(&self, component: &Self::C, property: UID) -> Option<serde_json::Value>;
}

pub(crate) enum ComponentKind {
    Static,
    Dynamic,
}

pub(crate) trait AnyComponentReflection {
    fn create_container(&self) -> Box<dyn AnyComponentContainer>;
    fn create_singleton(&self) -> Box<dyn AnyComponentSingleton>;
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
        self.define(name, ComponentKind::Static, Box::new(reflection))
    }

    pub(crate) fn define_dynamic(&mut self, name: &str) -> Result<UID, RegistryError> {
        let reflection = ComponentReflection::<DynamicComponent> { _phantom: std::marker::PhantomData };
        self.define(name, ComponentKind::Dynamic, Box::new(reflection))
    }

    pub(crate) fn get(&self, uid: UID) -> Option<&ComponentDefinition> {
        self.definitions.get(&uid)
    }
}