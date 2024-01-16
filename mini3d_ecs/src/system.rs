use core::cell::RefCell;

use alloc::{boxed::Box, rc::Rc, sync::Arc};
use mini3d_utils::{slot_map_key, slotmap::SlotMap};

use crate::{
    component::system::SystemKind,
    container::{ContainerEntry, ContainerTable},
    context::Context,
    entity::Entity,
    error::{ComponentError, SystemError},
    scheduler::NodeKey,
    world::World,
};

pub struct SystemResolver<'a> {
    containers: &'a mut ContainerTable,
    views: &'a [Entity],
    index: u8,
}

impl<'a> SystemResolver<'a> {
    pub(crate) fn next(&'_ mut self) -> Result<&'_ ContainerEntry, SystemError> {
        if self.index >= self.views.len() as u8 {
            return Err(SystemError::ConfigError);
        }
        let entity = self.views[self.index as usize];
        self.index += 1;
        let container = self
            .containers
            .entries
            .iter()
            .find_map(|(_, entry)| {
                if entry.entity == entity {
                    Some(entry)
                } else {
                    None
                }
            })
            .ok_or(SystemError::ConfigError)?;
        Ok(container)
    }
}

pub trait ExclusiveSystem: 'static {
    fn resolve(&mut self, resolver: SystemResolver) -> Result<(), SystemError>;
    fn run(&mut self, ctx: &mut Context) -> Result<(), SystemError>;
}

pub trait ParallelSystem: 'static {
    fn resolve(&mut self, resolver: SystemResolver) -> Result<(), SystemError>;
    fn run(&mut self, ctx: &Context) -> Result<(), SystemError>;
}

pub trait GlobalSystem: 'static {
    fn run(&mut self, ctx: &mut Context, world: &mut World) -> Result<(), SystemError>;
}

pub(crate) enum SystemInstance {
    Exclusive(Rc<RefCell<Box<dyn ExclusiveSystem>>>),
    Parallel(Arc<RefCell<Box<dyn ParallelSystem>>>),
    Global(Rc<RefCell<Box<dyn GlobalSystem>>>),
}

slot_map_key!(SystemKey);
slot_map_key!(SystemStageKey);

pub(crate) struct SystemEntry {
    pub(crate) entity: Entity,
    pub(crate) instance: SystemInstance,
    pub(crate) stage: SystemStageKey,
}

pub(crate) struct SystemStageEntry {
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
        if self.systems.iter().any(|(_, entry)| entry.entity == entity) {
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
        let instance = match system.kind {
            SystemKind::NativeExclusive { ref mut system, .. } => {
                SystemInstance::Exclusive(system.clone())
            }
            SystemKind::NativeParallel { ref mut system, .. } => {
                SystemInstance::Parallel(system.clone())
            }
            SystemKind::NativeGlobal(ref mut system) => SystemInstance::Global(system.clone()),
            SystemKind::Script => {
                return Err(ComponentError::UnresolvedReference);
            }
        };
        let key = self.systems.add(SystemEntry {
            entity,
            instance,
            stage,
        });
        Ok(())
    }

    pub(crate) fn disable_system(&mut self, entity: Entity, containers: &mut ContainerTable) {
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
        if self.stages.iter().any(|(_, entry)| entry.entity == entity) {
            return Err(ComponentError::DuplicatedEntry);
        }
        stage.key = self.stages.add(SystemStageEntry {
            entity,
            first_node: Default::default(),
        });
        Ok(())
    }

    pub(crate) fn disable_system_stage(&mut self, entity: Entity, containers: &mut ContainerTable) {
        unimplemented!()
    }
}
