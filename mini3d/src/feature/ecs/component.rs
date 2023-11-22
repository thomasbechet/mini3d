use mini3d_derive::{Reflect, Serialize};

use crate::{
    define_resource_handle,
    ecs::container::{native::single::NativeSingleContainer, Container, ContainerTable},
    feature::core::{resource::Resource, structure::StructDefinitionHandle},
    reflection::{Property, Reflect},
    resource::handle::ReferenceResolver,
    serialize::Serialize,
    slot_map_key,
};

slot_map_key!(ComponentKey);

pub struct ComponentContext {}

pub trait Component: 'static + Default + Reflect + Serialize {
    fn resolve_references(&mut self, references: &mut ReferenceResolver);
}

pub struct PrivateComponentTableRef<'a>(pub(crate) &'a ContainerTable);
pub struct PrivateComponentTableMut<'a>(pub(crate) &'a mut ContainerTable);

pub(crate) trait ComponentReflection {
    fn create_container(&self) -> Box<dyn Container>;
    fn find_property(&self, name: &str) -> Option<&Property>;
    fn properties(&self) -> &[Property];
}

pub(crate) struct NativeComponentReflection<C: Component> {
    pub(crate) _phantom: std::marker::PhantomData<C>,
}

impl<C: Component> ComponentReflection for NativeComponentReflection<C> {
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

#[derive(Serialize, Reflect, Default)]
pub(crate) enum ComponentKind {
    Native(#[serialize(skip)] Box<dyn ComponentReflection>),
    Struct(StructDefinitionHandle),
    Raw,
    #[default]
    Tag,
}

impl Default for Box<dyn ComponentReflection> {
    fn default() -> Self {
        panic!("Invalid deserialize for native component reflection")
    }
}

#[derive(Serialize, Default)]
pub enum ComponentStorage {
    #[default]
    Single,
    Array(usize),
    List,
    Map,
}

#[derive(Default, Serialize, Reflect)]
pub struct ComponentType {
    pub(crate) kind: ComponentKind,
    pub(crate) storage: ComponentStorage,
}

impl ComponentType {
    pub const NAME: &'static str = "RTY_ComponentType";

    pub fn native<C: Component>(storage: ComponentStorage) -> Self {
        let reflection = NativeComponentReflection::<C> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            kind: ComponentKind::Native(Box::new(reflection)),
            storage,
        }
    }

    pub fn structure(storage: ComponentStorage, structure: StructDefinitionHandle) -> Self {
        Self {
            kind: ComponentKind::Struct(structure),
            storage,
        }
    }

    pub(crate) fn create_container(&self) -> Box<dyn Container> {
        match &self.kind {
            ComponentKind::Native(reflection) => reflection.create_container(),
            _ => unimplemented!(),
        }
    }
}

impl Resource for ComponentType {}

define_resource_handle!(ComponentTypeHandle);
