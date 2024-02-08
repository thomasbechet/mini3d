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

pub trait NativeComponent: Serialize {
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
    fn on_post_added<Context>(
        _ecs: &mut ECS<Context>,
        _entity: Entity,
    ) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_post_removed<Context>(
        _ecs: &mut ECS<Context>,
        _entity: Entity,
    ) -> Result<(), ComponentError> {
        Ok(())
    }
}

pub type ComponentPostCallback<Context> =
    fn(&mut ECS<Context>, Entity) -> Result<(), ComponentError>;

pub trait RegisterComponent {
    fn register<Context>(ecs: &mut ECS<Context>) -> Result<Entity, ComponentError>;
}

impl<C: NativeComponent + NamedComponent> RegisterComponent for C {
    fn register<Context>(ecs: &mut ECS<Context>) -> Result<Entity, ComponentError> {
        let e = ecs.create();
        ecs.add(e, Identifier::new(C::IDENT));
        ecs.add(e, Component::single::<C>());
        Ok(e)
    }
}
