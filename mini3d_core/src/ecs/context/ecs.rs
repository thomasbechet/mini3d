use alloc::vec::Vec;

use crate::ecs::{
    entity::{Entity, EntityTable},
    scheduler::Scheduler,
    ECSCommand,
};

pub(crate) struct ECSContext<'a> {
    pub(crate) commands: &'a mut Vec<ECSCommand>,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) scheduler: &'a mut Scheduler,
    pub(crate) entity_created: &'a mut Vec<Entity>,
    pub(crate) entity_destroyed: &'a mut Vec<Entity>,
}
