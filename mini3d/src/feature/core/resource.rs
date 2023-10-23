use crate::{
    input::InputManager,
    reflection::{Property, Reflect},
    renderer::RendererManager,
    resource::{
        container::{NativeResourceContainer, ResourceContainer},
        handle::{ReferenceResolver, ResourceHandle},
        ResourceManager,
    },
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
    utils::{slotmap::SlotId, string::AsciiArray},
};

pub struct ResourceHookContext<'a> {
    pub input: &'a mut InputManager,
    pub renderer: &'a mut RendererManager,
    pub resource: &'a mut ResourceManager,
}

pub trait Resource: 'static + Default + Reflect {
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError>;
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver);
    fn hook_added(handle: ResourceHandle, ctx: ResourceHookContext) {}
    fn hook_removed(handle: ResourceHandle, ctx: ResourceHookContext) {}
}

pub(crate) trait ResourceReflection {
    fn create_resource_container(&self) -> Box<dyn ResourceContainer>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
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

pub(crate) enum ResourceKind {
    Native {
        reflection: Box<dyn ResourceReflection>,
    },
    Raw,
    Struct {
        structure: ResourceHandle,
    },
}

#[derive(Clone, Default)]
pub struct ResourceType {
    pub(crate) display_name: AsciiArray<32>,
    pub(crate) kind: ResourceKind,
    pub(crate) container_id: SlotId,
}

impl ResourceType {
    pub fn native<R: Resource>(name: &str) -> Self {
        let reflection = NativeResourceReflection::<R> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            display_name: AsciiArray::from_str(name),
            kind: ResourceKind::Native {
                reflection: Box::new(reflection),
            },
            container_id: SlotId::null(),
        }
    }

    pub fn structure(name: &str, structure: ResourceHandle) -> Self {
        Self {
            display_name: AsciiArray::from_str(name),
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
