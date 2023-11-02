use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{
    define_resource_handle,
    ecs::container::{native::single::NativeSingleContainer, Container, ContainerTable},
    reflection::{Property, Reflect},
    resource::handle::{ReferenceResolver, ResourceHandle},
    serialize::Serialize,
    utils::slotmap::SlotId,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ComponentId(pub(crate) SlotId);

pub struct ComponentContext {}

pub trait ComponentData: 'static + Default + Reflect + Serialize {
    fn on_create(&mut self, ctx: &mut ComponentContext);
    fn on_destroy(&mut self, ctx: &mut ComponentContext);
    fn resolve_references(&mut self, references: &mut ReferenceResolver);
}

pub struct PrivateComponentTableRef<'a>(pub(crate) &'a ContainerTable);
pub struct PrivateComponentTableMut<'a>(pub(crate) &'a mut ContainerTable);

pub(crate) trait ComponentReflection {
    fn create_container(&self) -> Box<dyn Container>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
}

pub(crate) struct NativeComponentReflection<C: ComponentData> {
    pub(crate) _phantom: std::marker::PhantomData<C>,
}

impl<C: ComponentData> ComponentReflection for NativeComponentReflection<C> {
    fn create_container(&self) -> Box<dyn Container> {
        Box::new(NativeSingleContainer::<C>::with_capacity(128))
    }

    fn find_property(&self, name: &str) -> Option<&Property> {
        C::PROPERTIES.iter().find(|p| p.name == name)
    }

    fn properties(&self) -> &[Property] {
        C::PROPERTIES
    }
}

pub(crate) enum ComponentKind {
    Native {
        reflection: Box<dyn ComponentReflection>,
    },
    Struct {
        structure: ResourceHandle,
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

#[derive(Clone, Default, Resource, Serialize, Reflect)]
pub struct Component {
    pub(crate) kind: ComponentKind,
    pub(crate) storage: ComponentStorage,
}

impl Component {
    pub const NAME: &'static str = "_component_type";

    pub fn native<C: ComponentData>(storage: ComponentStorage) -> Self {
        let reflection = NativeComponentReflection::<C> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            kind: ComponentKind::Native {
                reflection: Box::new(reflection),
            },
            storage,
        }
    }

    pub fn structure(storage: ComponentStorage, structure: ResourceHandle) -> Self {
        Self {
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

define_resource_handle!(ComponentHandle);
