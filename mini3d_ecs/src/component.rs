use core::any::Any;

use mini3d_serialize::Serialize;

use crate::{container::NativeContainer, ecs::ECS, entity::Entity, error::ComponentError};

use self::{component::Component, identifier::Identifier};

#[allow(clippy::module_inception)]
pub mod component;
pub mod identifier;
pub mod stage;
pub mod system;

pub struct EntityResolver;

impl EntityResolver {
    pub(crate) fn resolve(&mut self, entity: Entity) -> Entity {
        entity
    }
}

pub trait NamedComponent {
    const IDENT: &'static str;
}

pub trait NativeComponent: 'static + Default + Serialize {
    type Container: NativeContainer<Self>;
    fn resolve_entities(&mut self, resolver: &mut EntityResolver) -> Result<(), ComponentError> {
        resolver.resolve(Default::default());
        Ok(())
    }
    fn on_added(&mut self, _entity: Entity, _user: &mut dyn Any) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_removed(&mut self, _entity: Entity, _user: &mut dyn Any) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_post_added(_ecs: &mut ECS, _entity: Entity) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_post_removed(_ecs: &mut ECS, _entity: Entity) -> Result<(), ComponentError> {
        Ok(())
    }
}

pub type ComponentPostCallback = fn(&mut ECS, Entity) -> Result<(), ComponentError>;

pub trait RegisterComponent {
    fn register(ecs: &mut ECS) -> Result<Entity, ComponentError>;
}

impl<C: NativeComponent + NamedComponent> RegisterComponent for C {
    fn register(ecs: &mut ECS) -> Result<Entity, ComponentError> {
        let e = ecs.create();
        ecs.add(e, Identifier::new(C::IDENT));
        ecs.add(e, Component::single::<C>());
        Ok(e)
    }
}
