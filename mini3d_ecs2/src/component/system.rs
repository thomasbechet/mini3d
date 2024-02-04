use mini3d_derive::Serialize;

use crate::{container::linear::LinearContainer, ecs::ECS, entity::Entity, error::ComponentError};

use super::{NamedComponent, SingleComponent};

#[derive(Default, Clone, Serialize)]
pub enum SystemKind {
    Native(#[serialize(skip)] Option<fn(&mut ECS)>), // In option to allow serialization
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

impl SingleComponent for System {
    type Container = LinearContainer<Self>;

    fn on_post_added(ecs: &mut ECS, _entity: Entity) -> Result<(), ComponentError> {
        ecs.scheduler.rebuild(ecs.containers);
        Ok(())
    }

    fn on_post_removed(ecs: &mut ECS, _entity: Entity) -> Result<(), ComponentError> {
        ecs.scheduler.rebuild(ecs.containers);
        Ok(())
    }
}

impl System {
    pub fn native(callback: fn(&mut ECS), stage: Entity, order: SystemOrder) -> Self {
        Self {
            stage,
            order,
            kind: SystemKind::Native(Some(callback)),
        }
    }
}
