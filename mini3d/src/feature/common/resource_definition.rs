use crate::{
    registry::resource::{NativeResourceReflection, Resource, ResourceReflection},
    resource::handle::ResourceRef,
    utils::string::AsciiArray,
};

pub(crate) enum ResourceKind {
    Native {
        reflection: Box<dyn ResourceReflection>,
    },
    Raw,
    Struct {
        structure: ResourceRef,
    },
}

pub(crate) const MAX_RESOURCE_TYPE_NAME_LEN: usize = 64;

#[derive(Clone, Default)]
pub struct ResourceDefinition {
    pub(crate) name: AsciiArray<MAX_RESOURCE_TYPE_NAME_LEN>,
    pub(crate) kind: ResourceKind,
}

impl ResourceDefinition {
    pub fn native<R: Resource>(name: &str) -> Self {
        let reflection = NativeResourceReflection::<R> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            name: AsciiArray::from_str(name),
            kind: ResourceKind::Native {
                reflection: Box::new(reflection),
            },
        }
    }

    pub fn structure(name: &str, structure: ResourceRef) -> Self {
        Self {
            name: AsciiArray::from_str(name),
            kind: ResourceKind::Struct { structure },
        }
    }
}
