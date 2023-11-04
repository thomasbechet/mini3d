use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    feature::core::structure::StructDefinitionHandle,
    input::InputManager,
    reflection::{Property, Reflect},
    renderer::RendererManager,
    resource::{
        container::{NativeResourceContainer, ResourceContainer},
        handle::{ReferenceResolver, ResourceHandle},
        ResourceManager,
    },
    serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize},
    utils::slotmap::SlotId,
};

pub struct ResourceHookContext<'a> {
    pub input: &'a mut InputManager,
    pub renderer: &'a mut RendererManager,
    pub resource: &'a mut ResourceManager,
}

pub trait Resource: 'static + Default + Reflect + Serialize {
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver) {}
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {}
    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {}
}

pub(crate) trait ResourceReflection {
    fn create_resource_container(&self) -> Box<dyn ResourceContainer>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
}

impl Serialize for Box<dyn ResourceReflection> {
    type Header = ();
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        Ok(())
    }
    fn deserialize(
        decoder: &mut impl Decoder,
        header: &Self::Header,
    ) -> Result<Self, DecoderError> {
        panic!("Cannot deserialize ResourceReflection")
    }
}

pub(crate) struct NativeResourceReflection<R: Resource> {
    pub(crate) _phantom: std::marker::PhantomData<R>,
}

impl<R: Resource> ResourceReflection for NativeResourceReflection<R> {
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

#[derive(Default, Serialize, Reflect)]
pub(crate) enum ResourceKind {
    Native(Box<dyn ResourceReflection>),
    #[default]
    Raw,
    Struct(StructDefinitionHandle),
}

#[derive(Default, Serialize, Reflect)]
pub struct ResourceType {
    pub(crate) kind: ResourceKind,
    pub(crate) container: SlotId,
}

impl ResourceType {
    pub fn native<R: Resource>() -> Self {
        let reflection = NativeResourceReflection::<R> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            kind: ResourceKind::Native(Box::new(reflection)),
            container: SlotId::null(),
        }
    }

    pub fn structure(structure: StructDefinitionHandle) -> Self {
        Self {
            kind: ResourceKind::Struct(structure),
            container: SlotId::null(),
        }
    }

    pub(crate) fn create_container(&self) -> Box<dyn ResourceContainer> {
        match &self.kind {
            ResourceKind::Native(reflection) => reflection.create_resource_container(),
            ResourceKind::Raw => unimplemented!(),
            ResourceKind::Struct(structure) => unimplemented!(),
        }
    }
}

impl Resource for ResourceType {
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver) {
        match &mut self.kind {
            ResourceKind::Native(_) => {}
            ResourceKind::Raw => unimplemented!(),
            ResourceKind::Struct(structure) => {
                *structure = resolver.resolve_resource(*structure);
            }
        }
    }
}

define_resource_handle!(ResourceTypeHandle);
