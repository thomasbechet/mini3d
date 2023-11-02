use mini3d_derive::{Reflect, Serialize};

use crate::{
    input::InputManager,
    reflection::{Property, Reflect},
    renderer::RendererManager,
    resource::{
        container::{NativeResourceContainer, ResourceContainer},
        handle::{ReferenceResolver, ResourceHandle},
        ResourceManager,
    },
    serialize::Serialize,
    utils::slotmap::SlotId,
};

pub struct ResourceHookContext<'a> {
    pub input: &'a mut InputManager,
    pub renderer: &'a mut RendererManager,
    pub resource: &'a mut ResourceManager,
}

pub trait ResourceData: 'static + Default + Reflect + Serialize {
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver) {}
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {}
    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {}
}

pub(crate) trait ResourceReflection {
    fn create_resource_container(&self) -> Box<dyn ResourceContainer>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
}

pub(crate) struct NativeResourceReflection<R: ResourceData> {
    pub(crate) _phantom: std::marker::PhantomData<R>,
}

impl<R: ResourceData> ResourceReflection for NativeResourceReflection<R> {
    fn create_resource_container(&self) -> Box<dyn ResourceContainer> {
        Box::new(NativeResourceContainer::<R>::with_capacity(128))
    }

    fn find_property(&self, name: &str) -> Option<&Property> {
        R::PROPERTIES.iter().find(|p| p.name == name)
    }

    fn properties(&self) -> &[Property] {
        R::PROPERTIES
    }
}

pub(crate) enum ResourceKind {
    Native {
        reflection: Box<dyn ResourceReflection>,
    },
    Raw,
    Struct {
        structure: ResourceHandle,
    },
}

#[derive(Clone, Default, Serialize, Debug, Reflect)]
pub struct Resource {
    pub(crate) kind: ResourceKind,
    pub(crate) container_id: SlotId,
}

impl Resource {
    pub fn native<R: ResourceData>() -> Self {
        let reflection = NativeResourceReflection::<R> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            kind: ResourceKind::Native {
                reflection: Box::new(reflection),
            },
            container_id: SlotId::null(),
        }
    }

    pub fn structure(structure: ResourceHandle) -> Self {
        Self {
            kind: ResourceKind::Struct { structure },
            container_id: SlotId::null(),
        }
    }

    pub(crate) fn create_container(&self) -> Box<dyn ResourceContainer> {
        match &self.kind {
            ResourceKind::Native { reflection } => reflection.create_resource_container(),
            ResourceKind::Raw => unimplemented!(),
            ResourceKind::Struct { structure } => unimplemented!(),
        }
    }
}
