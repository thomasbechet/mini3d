use std::any::TypeId;

use crate::{
    ecs::container::{
        native::single::{NativeSingleContainer, SingleContainer},
        Container, ContainerTable,
    },
    reflection::{Property, Reflect},
    resource::handle::{ReferenceResolver, ResourceRef},
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
    utils::{slotmap::SlotId, string::AsciiArray},
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub(crate) SlotId);

pub struct ComponentContext<'a> {}

pub trait Component: 'static + Default + Reflect {
    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError>;
    fn deserialize(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError>;
    fn on_create(&mut self, ctx: &mut ComponentContext);
    fn on_destroy(&mut self, ctx: &mut ComponentContext);
    fn resolve_references(&mut self, references: &mut ReferenceResolver);
}

pub struct PrivateComponentTableRef<'a>(pub(crate) &'a ContainerTable);
pub struct PrivateComponentTableMut<'a>(pub(crate) &'a mut ContainerTable);

pub(crate) trait ComponentReflection {
    fn create_container(&self) -> Box<dyn SingleContainer>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
    fn type_id(&self) -> TypeId;
}

pub(crate) struct NativeComponentReflection<C: Component> {
    pub(crate) _phantom: std::marker::PhantomData<C>,
}

impl<C: Component> ComponentReflection for NativeComponentReflection<C> {
    fn create_container(&self) -> Box<dyn SingleContainer> {
        Box::new(NativeSingleContainer::<C>::with_capacity(128))
    }

    fn find_property(&self, name: &str) -> Option<&Property> {
        C::PROPERTIES.iter().find(|p| p.name == name)
    }

    fn properties(&self) -> &[Property] {
        C::PROPERTIES
    }

    fn type_id(&self) -> TypeId {
        TypeId::of::<C>()
    }
}

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

#[derive(Clone, Default)]
pub struct ComponentType {
    pub(crate) access_name: AsciiArray<32>,
    pub(crate) kind: ComponentKind,
    pub(crate) storage: ComponentStorage,
}

impl ComponentType {
    pub fn native<C: Component>(access_name: &str, storage: ComponentStorage) -> Self {
        let reflection = NativeComponentReflection::<C> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            access_name: AsciiArray::from_str(access_name),
            kind: ComponentKind::Native {
                reflection: Box::new(reflection),
            },
            storage,
        }
    }

    pub fn structure(access_name: &str, storage: ComponentStorage, structure: ResourceRef) -> Self {
        Self {
            access_name: AsciiArray::from_str(access_name),
            kind: ComponentKind::Struct { structure },
            storage,
        }
    }

    pub(crate) fn create_container(&self) -> Box<dyn Container> {
        match &self.kind {
            ComponentKind::Native { reflection } => reflection.create_container(),
            _ => unimplemented!(),
        }
    }
}
