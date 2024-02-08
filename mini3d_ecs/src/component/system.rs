use core::any::Any;

use alloc::boxed::Box;
use mini3d_derive::Serialize;

use crate::{container::linear::LinearContainer, ecs::ECS, entity::Entity, error::ComponentError};

use super::{NamedComponent, NativeComponent};

#[derive(Default, Serialize)]
pub enum SystemKind {
    Native(#[serialize(skip)] Option<Box<dyn Any>>), // In option to allow serialization
    #[default]
    Script,
}

#[derive(Default, Serialize)]
pub struct SystemOrder {}

#[derive(Default, Serialize)]
pub struct System {
    pub(crate) stage: Entity,
    pub(crate) order: SystemOrder,
    pub(crate) kind: SystemKind,
}

impl NamedComponent for System {
    const IDENT: &'static str = "system";
}

impl NativeComponent for System {
    type Container = LinearContainer<Self>;

    fn on_post_added<Context>(
        ecs: &mut ECS<Context>,
        _entity: Entity,
    ) -> Result<(), ComponentError> {
        ecs.scheduler.rebuild(ecs.containers);
        Ok(())
    }

    fn on_post_removed<Context>(
        ecs: &mut ECS<Context>,
        _entity: Entity,
    ) -> Result<(), ComponentError> {
        ecs.scheduler.rebuild(ecs.containers);
        Ok(())
    }
}

impl System {
    pub fn native<Context>(
        callback: fn(&mut ECS<Context>),
        stage: Entity,
        order: SystemOrder,
    ) -> Self {
        Self {
            stage,
            order,
            kind: SystemKind::Native(Some(Box::new(callback))),
        }
    }
}
