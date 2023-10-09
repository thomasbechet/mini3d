use crate::{
    registry::component::{Component, ComponentReflection, NativeComponentReflection},
    resource::handle::ResourceRef,
    utils::string::AsciiArray,
};

pub(crate) enum ComponentKind {
    Native {
        reflection: Box<dyn ComponentReflection>,
    },
    Struct {
        structure: ResourceRef,
    },
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
}

impl ComponentDefinition {
    pub fn native<C: Component>(name: &str, storage: ComponentStorage) -> Self {
        let reflection = NativeComponentReflection::<C> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            name: AsciiArray::from_str(name),
            kind: ComponentKind::Native {
                reflection: Box::new(reflection),
            },
            storage,
        }
    }

    pub fn structure(name: &str, storage: ComponentStorage, structure: ResourceRef) -> Self {
        Self {
            name: AsciiArray::from_str(name),
            kind: ComponentKind::Struct { structure },
            storage,
        }
    }
}
