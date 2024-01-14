use core::cell::RefCell;

use alloc::{boxed::Box, rc::Rc};
use mini3d_utils::{
    slot_map_key,
    slotmap::SlotMap,
    uid::{ToUID, UID},
};

use crate::{
    component::system::SystemKind,
    container::ContainerTable,
    context::Context,
    entity::Entity,
    error::{ComponentError, SystemError},
    scheduler::NodeKey,
};

pub trait ExclusiveSystem {
    fn configure(&mut self) -> Result<(), SystemError>;
    fn run(&mut self, ctx: &mut Context) -> Result<(), SystemError>;
}

pub trait ParallelSystem {
    fn configure(&mut self) -> Result<(), SystemError>;
    fn run(&mut self, ctx: &Context) -> Result<(), SystemError>;
}

pub(crate) enum SystemInstance {
    Exclusive(Rc<RefCell<Box<dyn ExclusiveSystem>>>),
    Parallel(Rc<RefCell<Box<dyn ParallelSystem>>>),
}

slot_map_key!(SystemKey);
slot_map_key!(SystemStageKey);

pub(crate) struct SystemEntry {
    pub(crate) uid: UID,
    pub(crate) instance: SystemInstance,
    pub(crate) stage: SystemStageKey,
}

pub(crate) struct SystemStageEntry {
    pub(crate) uid: UID,
    pub(crate) entity: Entity,
    pub(crate) first_node: NodeKey,
}

#[derive(Default)]
pub(crate) struct SystemTable {
    pub(crate) systems: SlotMap<SystemKey, SystemEntry>,
    pub(crate) stages: SlotMap<SystemStageKey, SystemStageEntry>,
}

impl SystemTable {
    pub(crate) fn enable_system(
        &mut self,
        entity: Entity,
        containers: &mut ContainerTable,
    ) -> Result<(), ComponentError> {
        let container = containers.system_container();
        let system = container
            .get_mut(entity)
            .ok_or(ComponentError::EntryNotFound)?;
        let uid = system.name.to_uid();
        if self.systems.iter().any(|(_, entry)| entry.uid == uid) {
            return Err(ComponentError::DuplicatedEntry);
        }
        let stage = self
            .stages
            .iter()
            .find_map(|(key, entry)| {
                if entry.entity == system.stage {
                    Some(key)
                } else {
                    None
                }
            })
            .ok_or(ComponentError::UnresolvedReference)?;
        match system.kind {
            SystemKind::NativeExclusive(ref mut system) => {}
            SystemKind::NativeParallel(ref mut system) => {}
            SystemKind::Script => {}
        }
        Ok(self.systems.add(SystemEntry {
            instance,
            stage,
            uid,
        }))
    }

    pub(crate) fn disable_system(&mut self, key: SystemKey) {
        unimplemented!()
    }

    pub(crate) fn enable_system_stage(
        &mut self,
        entity: Entity,
        containers: &mut ContainerTable,
    ) -> Result<(), ComponentError> {
        let container = containers.system_stage_container();
        let stage = container
            .get_mut(entity)
            .ok_or(ComponentError::EntryNotFound)?;
        let uid = stage.name.to_uid();
        if self.stages.iter().any(|(_, entry)| entry.uid == uid) {
            return Err(ComponentError::DuplicatedEntry);
        }
        stage.key = self.stages.add(SystemStageEntry {
            uid,
            entity,
            first_node: Default::default(),
        });
        Ok(())
    }

    pub(crate) fn disable_system_stage(&mut self, entity: Entity) {
        unimplemented!()
    }
}
