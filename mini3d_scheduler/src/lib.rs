#![no_std]

use alloc::vec::Vec;
use mini3d_derive::{Error, Serialize};
use mini3d_utils::{
    slot_map_key,
    slotmap::{Key, SlotMap},
    string::AsciiArray,
};

#[cfg(test)]
extern crate std;

extern crate alloc;

#[derive(Error, Debug)]
pub enum SchedulerError {
    #[error("Duplicated entry")]
    DuplicatedEntry,
}

slot_map_key!(NodeId);
slot_map_key!(StageId);
slot_map_key!(SystemId);

#[derive(Default, Serialize)]
pub struct SystemOrder {}

#[derive(Serialize)]
pub struct System {
    pub name: AsciiArray<32>,
    pub(crate) stage: StageId,
    pub(crate) order: SystemOrder,
}

pub struct Stage {
    pub name: AsciiArray<32>,
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
    stages: SlotMap<StageId, Stage>,
    systems: SlotMap<SystemId, System>,
    indices: Vec<SystemId>,
}

impl Scheduler {
    pub fn rebuild(&mut self) {
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

    pub fn first_node(&self, stage: StageId) -> Option<NodeId> {
        self.stages.get(stage).and_then(|stage| stage.first_node)
    }

    pub fn next_node(&self, node: NodeId) -> Option<NodeId> {
        self.nodes[node].next
    }

    pub fn systems(&self, node: NodeId) -> &'_ [SystemId] {
        let node = self.nodes[node];
        &self.indices[node.first as usize..(node.first + node.count) as usize]
    }

    pub fn find_stage(&self, name: &str) -> Option<StageId> {
        self.stages.iter().find_map(|(id, stage)| {
            if stage.name.as_str() == name {
                Some(id)
            } else {
                None
            }
        })
    }

    pub fn find_system(&self, name: &str) -> Option<SystemId> {
        self.systems.iter().find_map(|(id, system)| {
            if system.name.as_str() == name {
                Some(id)
            } else {
                None
            }
        })
    }

    pub fn add_stage(&mut self, name: &str) -> Result<StageId, SchedulerError> {
        if self.find_stage(name).is_some() {
            return Err(SchedulerError::DuplicatedEntry);
        }
        Ok(self.stages.add(Stage {
            name: AsciiArray::from(name),
            first_node: None,
        }))
    }

    pub fn add_system(
        &mut self,
        name: &str,
        stage: StageId,
        order: SystemOrder,
    ) -> Result<SystemId, SchedulerError> {
        if self.find_system(name).is_some() {
            return Err(SchedulerError::DuplicatedEntry);
        }
        Ok(self.systems.add(System {
            name: AsciiArray::from(name),
            stage,
            order,
        }))
    }

    pub fn iter_stages(&self) -> impl Iterator<Item = StageId> + '_ {
        self.stages.keys()
    }
    
    pub fn stage(&self, id: StageId) -> Option<&Stage> {
        self.stages.get(id)
    }

    pub fn system(&self, id: SystemId) -> Option<&System> {
        self.systems.get(id)
    }
}
