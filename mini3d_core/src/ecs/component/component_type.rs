use alloc::boxed::Box;
use mini3d_derive::{Error, Reflect, Serialize};

use crate::{
    ecs::{
        container::{native::single::NativeSingleContainer, Container, ContainerTable},
        context::Context,
        entity::Entity,
    },
    input::InputManager,
    reflection::Reflect,
    renderer::RendererManager,
    serialize::Serialize,
    slot_map_key,
    utils::string::AsciiArray,
};

slot_map_key!(ComponentKey);

pub struct EntityResolver;

impl EntityResolver {
    pub(crate) fn resolve(&mut self, entity: Entity) -> Entity {
        Default::default()
    }
}

pub struct ComponentContext<'a> {
    pub input: &'a mut InputManager,
    pub renderer: &'a mut RendererManager,
}

#[derive(Error)]
pub enum ComponentError {
    #[error("Component did not match unicity constraint")]
    DuplicatedEntry,
    #[error("Component reference not found")]
    UnresolvedReference,
    #[error("Component provider error")]
    ProviderError,
}

pub trait Component: 'static + Default + Reflect + Serialize {
    const STORAGE: ComponentStorage;
    fn resolve_entities(&mut self, resolver: &mut EntityResolver) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_added(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_removed(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {
        Ok(())
    }
}

pub struct PrivateComponentTableRef<'a>(pub(crate) &'a ContainerTable);
pub struct PrivateComponentTableMut<'a>(pub(crate) &'a mut ContainerTable);

trait ComponentFactory {
    fn create_container(&self) -> Box<dyn Container>;
}

struct NativeComponentFactory<C: Component>(core::marker::PhantomData<C>);

impl<C: Component> ComponentFactory for NativeComponentFactory<C> {
    fn create_container(&self) -> Box<dyn Container> {
        match C::STORAGE {
            ComponentStorage::Single => Box::new(NativeSingleContainer::<C>::with_capacity(128)),
            _ => unimplemented!(),
        }
    }
}

#[derive(Serialize, Reflect, Default)]
pub(crate) enum ComponentKind {
    Native(#[serialize(skip)] Box<dyn ComponentFactory>),
    Dynamic {
        typedef: Entity,
        storage: ComponentStorage,
    },
    Raw {
        storage: ComponentStorage,
    },
    #[default]
    Tag,
}

impl Default for Box<dyn ComponentFactory> {
    fn default() -> Self {
        panic!("Invalid deserialize for native component")
    }
}

#[derive(Serialize, Default)]
pub enum ComponentStorage {
    #[default]
    Single,
    Array(usize),
    List,
    Map,
    Spatial,
}

#[derive(Default, Serialize, Reflect)]
pub struct ComponentType {
    pub(crate) name: AsciiArray<32>,
    pub(crate) kind: ComponentKind,
    pub(crate) enable_default: bool,
}

impl ComponentType {
    pub const NAME: &'static str = "component_type";

    pub fn native<C: Component>(enable: bool) -> Self {
        Self {
            name: AsciiArray::from(C::NAME),
            kind: ComponentKind::Native(Box::new(NativeComponentFactory::<C>(
                core::marker::PhantomData,
            ))),
            enable_default: enable,
        }
    }

    pub fn dynamic(name: &str, storage: ComponentStorage, typedef: Entity, enable: bool) -> Self {
        Self {
            name: AsciiArray::from(name),
            kind: ComponentKind::Dynamic { typedef, storage },
            enable_default: enable,
        }
    }

    pub(crate) fn create_container(&self) -> Box<dyn Container> {
        match &self.kind {
            ComponentKind::Native(reflection) => reflection.create_container(),
            _ => unimplemented!(),
        }
    }

    pub fn enable(ctx: &mut Context, ty: Entity) {}

    pub fn disable(ctx: &mut Context, ty: Entity) {}
}

impl Component for ComponentType {
    const STORAGE: ComponentStorage = ComponentStorage::Single;
}
