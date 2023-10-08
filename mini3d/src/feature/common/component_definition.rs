use crate::{
    registry::component::{Component, ComponentReflection, NativeComponentReflection},
    utils::string::AsciiArray,
};

pub(crate) enum ComponentKind {
    Native,
    Struct,
    Raw,
    Tag,
}

pub enum ComponentStorage {
    Single,
    Array(usize),
    List,
    Map,
}

pub const MAX_COMPONENT_NAME_LEN: usize = 32;

#[derive(Clone, Default)]
pub struct ComponentDefinition {
    pub(crate) name: AsciiArray<MAX_COMPONENT_NAME_LEN>,
    pub(crate) kind: ComponentKind,
    pub(crate) storage: ComponentStorage,
    pub(crate) reflection: Box<dyn ComponentReflection>,
}

impl ComponentDefinition {
    pub fn native<C: Component>(name: &str, storage: ComponentStorage) -> Self {
        let reflection = NativeComponentReflection::<C> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            name: AsciiArray::from_str(name),
            kind: ComponentKind::Native,
            storage,
            reflection: Box::new(reflection),
        }
    }
}
