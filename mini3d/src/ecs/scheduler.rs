use std::collections::HashMap;
use core::cell::RefCell;

use mini3d_derive::Serialize;

use crate::{uid::UID, feature::asset::system_group::SystemGroup, registry::{RegistryManager, error::RegistryError}};

use super::{pipeline::CompiledSystemPipeline, error::SchedulerError};

pub enum Invocation {
    Immediate,
    EndFrame,
    NextFrame,
}

#[derive(Serialize)]
struct SystemGroupEntry {
    group: SystemGroup,
    enabled: bool,
}

#[derive(Serialize)]
struct ProcedureEntry {
    name: String,
    groups: Vec<(UID, i32)>,
}

impl ProcedureEntry {
    fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            groups: Vec::new(),
        }
    }
}

#[derive(Default, Serialize)]
pub(crate) struct Scheduler {
    groups: HashMap<UID, SystemGroupEntry>,
    procedures: HashMap<UID, ProcedureEntry>,
}

impl Scheduler {

    pub(crate) fn build_pipeline(&self, procedure: UID, registry: &RefCell<RegistryManager>) -> Result<Option<CompiledSystemPipeline>, RegistryError> {
        if let Some(entry) = self.procedures.get(&procedure) {
            return Ok(Some(CompiledSystemPipeline::build(&registry.borrow().systems, entry.groups.iter()
                .map(|(group, _)| self.groups.get(group).unwrap())
                .filter(|group| group.enabled)
                .flat_map(move |group| group.group.procedures.get(&procedure).unwrap().pipeline.systems.iter()))?));
        }
        Ok(None)
    }

    pub(crate) fn add_group(&mut self, name: &str, group: SystemGroup) -> Result<UID, SchedulerError> {
        let uid: UID = name.into();
        // Check existing group
        if self.groups.contains_key(&uid) {
            return Err(SchedulerError::DuplicatedGroup { name: name.to_owned() });
        }
        // Insert procedures
        for (procedure_uid, procedure) in &group.procedures {
            let procedures = self.procedures.entry(*procedure_uid).or_insert_with(|| ProcedureEntry::new(&procedure.name));
            procedures.groups.push((uid, procedure.priority));
            procedures.groups.sort_by_key(|(_, priority)| *priority);
        }
        // Insert group
        self.groups.insert(uid, SystemGroupEntry { group, enabled: true });
        Ok(uid)
    }

    pub(crate) fn remove_group(&mut self, group: UID) -> Result<(), SchedulerError> {
        if self.groups.remove(&group).is_none() {
            return Err(SchedulerError::GroupNotFound { uid: group });
        }
        self.procedures.iter_mut().for_each(|(_, procedure)| {
            procedure.groups.retain(|(group_uid, _)| group_uid != &group)
        });
        Ok(())
    }

    pub(crate) fn enable_group(&mut self, group: UID) -> Result<(), SchedulerError> {
        self.groups.get_mut(&group).ok_or(SchedulerError::GroupNotFound { uid: group })?.enabled = true;
        Ok(())
    }

    pub(crate) fn disable_group(&mut self, group: UID) -> Result<(), SchedulerError> {
        self.groups.get_mut(&group).ok_or(SchedulerError::GroupNotFound { uid: group })?.enabled = false;
        Ok(())
    }
}