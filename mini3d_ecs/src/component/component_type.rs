use core::cell::UnsafeCell;

use alloc::boxed::Box;
use mini3d_derive::Serialize;
use mini3d_utils::slotmap::Key;

use crate::{
    container::{native::NativeSingleContainer, ContainerKey, ContainerWrapper},
    context::{Command, Context},
    entity::Entity,
    error::ComponentError,
};

use super::{Component, ComponentStorage, EntityResolver};

pub trait ComponentFactory {
    fn create_container(&self) -> ContainerWrapper;
}

struct NativeComponentFactory<C: Component>(core::marker::PhantomData<C>);

impl<C: Component> ComponentFactory for NativeComponentFactory<C> {
    fn create_container(&self) -> ContainerWrapper {
        match C::STORAGE {
            ComponentStorage::Single => Box::new(UnsafeCell::new(
                NativeSingleContainer::<C>::with_capacity(128),
            )),
            _ => unimplemented!(),
        }
    }
}

impl Default for Box<dyn ComponentFactory> {
    fn default() -> Self {
        panic!("Invalid deserialization for native component")
    }
}

#[derive(Default, Serialize)]
pub(crate) enum ComponentKind {
    Native(#[serialize(skip)] Box<dyn ComponentFactory>),
    Dynamic {
        storage: ComponentStorage,
    },
    Raw {
        storage: ComponentStorage,
    },
    #[default]
    Tag,
}

#[derive(Default, Serialize)]
pub struct ComponentType {
    pub(crate) kind: ComponentKind,
    pub(crate) auto_enable: bool,
    #[serialize(skip)]
    pub(crate) key: ContainerKey,
}

impl ComponentType {
    pub const NAME: &'static str = "component_type";

    pub fn native<C: Component>(auto_enable: bool) -> Self {
        Self {
            kind: ComponentKind::Native(Box::new(NativeComponentFactory::<C>(
                core::marker::PhantomData,
            ))),
            auto_enable,
            key: Default::default(),
        }
    }

    pub fn dynamic(storage: ComponentStorage, typedef: Entity, auto_enable: bool) -> Self {
        Self {
            kind: ComponentKind::Dynamic { storage },
            auto_enable,
            key: Default::default(),
        }
    }

    pub(crate) fn create_container(&self) -> ContainerWrapper {
        match &self.kind {
            ComponentKind::Native(reflection) => reflection.create_container(),
            _ => unimplemented!(),
        }
    }

    pub fn is_enable(&self) -> bool {
        !self.key.is_null()
    }
}

impl Component for ComponentType {
    const STORAGE: ComponentStorage = ComponentStorage::Single;

    fn resolve_entities(&mut self, resolver: &mut EntityResolver) -> Result<(), ComponentError> {
        Ok(())
    }

    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if self.auto_enable {
            Command::enable_component_type(ctx, entity);
        }
        Ok(())
    }

    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if !self.key.is_null() {
            Command::disable_component_type(ctx, entity);
        }
        Ok(())
    }
}
