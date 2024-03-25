#![no_std]

use alloc::vec::Vec;
use mini3d_derive::{Error, Serialize};
use mini3d_utils::{slot_map_key, slotmap::SlotMap, string::AsciiArray};

#[cfg(test)]
extern crate std;

extern crate alloc;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Duplicated entry")]
    DuplicatedEntry,
}

slot_map_key!(NodeId);
slot_map_key!(StageHandle);
slot_map_key!(SystemHandle);

#[derive(Default, Debug, Serialize)]
pub struct SystemOrder {}

#[derive(Default, Debug, Serialize, PartialEq, Eq)]
pub enum RegisterItemState {
    #[default]
    Created,
    Running,
    Deleted,
}

#[derive(Serialize, Debug)]
pub struct System {
    pub name: AsciiArray<32>,
    pub state: RegisterItemState,
    pub active: bool,
    pub(crate) stage: StageHandle,
    pub(crate) order: SystemOrder,
}

pub struct Stage {
    pub name: AsciiArray<32>,
    pub state: RegisterItemState,
    pub(crate) first_node: Option<NodeId>,
}

#[derive(Clone, Copy)]
pub(crate) struct PipelineNode {
    first: u16, // Offset in instance indices
    count: u16, // Number of instances
    next: Option<NodeId>,
}

#[derive(Default)]
pub struct Scheduler {
    nodes: SlotMap<NodeId, PipelineNode>,
    stages: SlotMap<StageHandle, Stage>,
    systems: SlotMap<SystemHandle, System>,
    indices: Vec<SystemHandle>,
}

impl Scheduler {
    pub fn rebuild(&mut self) {
        // Update registry
        for id in self.stages_from_state(RegisterItemState::Deleted) {
            self.stages.remove(id);
        }
        for id in self.stages_from_state(RegisterItemState::Created) {
            self.stages[id].state = RegisterItemState::Running;
        }
        for id in self.systems_from_state(RegisterItemState::Deleted) {
            self.systems.remove(id);
        }
        for id in self.systems_from_state(RegisterItemState::Created) {
            self.systems[id].state = RegisterItemState::Running;
        }

        // Reset baked resources
        self.nodes.clear();
        self.indices.clear();

        // Reset stage entry nodes
        for stage in self.stages.values_mut() {
            stage.first_node = None;
        }

        // Collect stages
        let stages = self.stages.keys().collect::<Vec<_>>();
        for stage in stages {
            // Collect systems in stage
            let start_index = self.indices.len();
            for system in self.systems.iter() {
                if system.1.stage == stage {
                    self.indices.push(system.0);
                }
            }
            let stop_index = self.indices.len();

            // TODO: apply ordering

            // Build nodes
            let mut previous_node = None;
            for (index, _) in self.indices[start_index..stop_index].iter().enumerate() {
                // TODO: detect parallel nodes

                // Create exclusive node
                let node = self.nodes.add(PipelineNode {
                    first: (start_index + index) as u16,
                    count: 1,
                    next: Default::default(),
                });

                // Link previous node or create new stage
                if let Some(previous_node) = previous_node {
                    self.nodes[previous_node].next = Some(node);
                } else {
                    // Update stage first node
                    self.stages.get_mut(stage).unwrap().first_node = Some(node);
                }

                // Next previous node
                previous_node = Some(node);
            }
        }
    }

    pub fn first_node(&self, stage: StageHandle) -> Option<NodeId> {
        self.stages.get(stage).and_then(|stage| stage.first_node)
    }

    pub fn next_node(&self, node: NodeId) -> Option<NodeId> {
        self.nodes[node].next
    }

    pub fn systems(&self, node: NodeId) -> &'_ [SystemHandle] {
        let node = self.nodes[node];
        &self.indices[node.first as usize..(node.first + node.count) as usize]
    }

    pub fn find_stage(&self, name: &str) -> Option<StageHandle> {
        self.stages.iter().find_map(|(id, stage)| {
            if stage.name.as_str() == name && stage.state != RegisterItemState::Deleted {
                Some(id)
            } else {
                None
            }
        })
    }

    pub fn find_system(&self, name: &str) -> Option<SystemHandle> {
        self.systems.iter().find_map(|(id, system)| {
            if system.name.as_str() == name && system.state != RegisterItemState::Deleted {
                Some(id)
            } else {
                None
            }
        })
    }

    pub fn add_stage(&mut self, name: &str) -> Result<StageHandle, SchedulerError> {
        if self.find_stage(name).is_some() {
            return Err(SchedulerError::DuplicatedEntry);
        }
        Ok(self.stages.add(Stage {
            name: AsciiArray::from(name),
            state: RegisterItemState::Created,
            first_node: None,
        }))
    }

    pub fn add_system(
        &mut self,
        name: &str,
        stage: StageHandle,
        order: SystemOrder,
    ) -> Result<SystemHandle, SchedulerError> {
        if self.find_system(name).is_some() {
            return Err(SchedulerError::DuplicatedEntry);
        }
        Ok(self.systems.add(System {
            name: AsciiArray::from(name),
            state: RegisterItemState::Created,
            active: true,
            stage,
            order,
        }))
    }

    pub fn remove_system(&mut self, id: SystemHandle) {
        self.systems[id].state = RegisterItemState::Deleted;
    }

    pub fn remove_stage(&mut self, id: StageHandle) {
        self.stages[id].state = RegisterItemState::Deleted;
    }

    pub fn iter_stages(&self) -> impl Iterator<Item = StageHandle> + '_ {
        self.stages.keys()
    }

    pub fn iter_systems(&self) -> impl Iterator<Item = SystemHandle> + '_ {
        self.systems.keys()
    }

    pub fn stage(&self, id: StageHandle) -> Option<&Stage> {
        self.stages.get(id)
    }

    pub fn system(&self, id: SystemHandle) -> Option<&System> {
        self.systems.get(id)
    }

    pub fn systems_from_state(&self, state: RegisterItemState) -> Vec<SystemHandle> {
        self.systems
            .iter()
            .filter_map(|(id, system)| {
                if system.state == state {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn stages_from_state(&self, state: RegisterItemState) -> Vec<StageHandle> {
        self.stages
            .iter()
            .filter_map(|(id, system)| {
                if system.state == state {
                    Some(id)
                } else {
                    None
                }
            })
            .collect()
    }
}
