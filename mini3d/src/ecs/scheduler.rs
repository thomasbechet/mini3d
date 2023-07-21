use core::cell::RefCell;
use std::collections::HashMap;

use mini3d_derive::Serialize;

use crate::{
    feature::component::common::system_graph::SystemGraph,
    registry::{error::RegistryError, RegistryManager},
    utils::uid::UID,
};

use super::{error::SchedulerError, pipeline::SystemPipeline};

pub enum Invocation {
    Immediate,
    EndFrame,
    NextFrame,
}

struct SignalEntry {
    name: String,
    pipeline: SystemPipeline,
}

#[derive(Default, Serialize)]
pub(crate) struct SystemGraphEntry {
    graph: SystemGraph,
    priority: i32,
}

#[derive(Default)]
pub(crate) struct Scheduler {
    signals: HashMap<UID, SignalEntry>,
    graphs: HashMap<UID, SystemGraphEntry>,
}

impl Scheduler {
    // pub(crate) fn build_pipeline(
    //     &mut self,
    //     procedure: UID,
    //     registry: &RefCell<RegistryManager>,
    // ) -> Result<Option<SystemPipeline>, RegistryError> {
    //     if let Some(entry) = self.procedures.get(&procedure) {
    //         return Ok(Some(SystemPipeline::build(
    //             &registry.borrow().systems,
    //             entry
    //                 .groups
    //                 .iter()
    //                 .map(|(group, _)| self.groups.get(group).unwrap())
    //                 .filter(|group| group.enabled)
    //                 .flat_map(move |group| {
    //                     group
    //                         .group
    //                         .procedures
    //                         .get(&procedure)
    //                         .unwrap()
    //                         .pipeline
    //                         .systems
    //                         .iter()
    //                 }),
    //         )?));
    //     }
    //     Ok(None)
    // }

    pub(crate) fn build_pipelines(&mut self) -> Result<(), SchedulerError> {
        Ok(())
    }

    pub(crate) fn add_group(
        &mut self,
        name: &str,
        group: SystemGroup,
    ) -> Result<UID, SchedulerError> {
        let uid: UID = name.into();
        // Check existing group
        if self.groups.contains_key(&uid) {
            return Err(SchedulerError::DuplicatedGroup {
                name: name.to_owned(),
            });
        }
        // Insert procedures
        for (signal_uid, signal) in &group.procedures {
            let signals = self
                .signals
                .entry(*signal_uid)
                .or_insert_with(|| SignalEntry::new(&signal.name));
            signals.groups.push((uid, signal.priority));
            signals.groups.sort_by_key(|(_, priority)| *priority);
        }
        // Insert group
        self.groups.insert(
            uid,
            SystemGroupEntry {
                group,
                enabled: true,
            },
        );
        Ok(uid)
    }

    pub(crate) fn enable_group(&mut self, group: UID) -> Result<(), SchedulerError> {
        self.groups
            .get_mut(&group)
            .ok_or(SchedulerError::GroupNotFound { uid: group })?
            .enabled = true;
        Ok(())
    }

    pub(crate) fn disable_group(&mut self, group: UID) -> Result<(), SchedulerError> {
        self.groups
            .get_mut(&group)
            .ok_or(SchedulerError::GroupNotFound { uid: group })?
            .enabled = false;
        Ok(())
    }
}
