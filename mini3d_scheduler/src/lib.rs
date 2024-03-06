#![no_std]

use alloc::{vec::Vec};
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

#[derive(Default, Serialize)]
pub struct System {
    pub(crate) name: AsciiArray<32>,
    pub(crate) stage: StageId,
    pub(crate) order: SystemOrder,
}

pub(crate) struct Stage {
    pub(crate) name: AsciiArray<32>,
    pub(crate) first_node: NodeId,
}

#[derive(Clone, Copy)]
pub(crate) struct PipelineNode {
    first: u16, // Offset in instance indices
    count: u16, // Number of instances
    next: NodeId,
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
            stage.first_node = NodeId::null();
        }

        // Collect stages
        let stages = self.stages.keys().collect::<Vec<_>>();
        for stage in stages {
            // Collect systems in stage
            for system in self.systems.iter() {
                if system.1.stage == stage {
                    self.indices.push(system.0);
                }
            }

            // TODO: apply ordering

            // Build nodes
            let mut previous_node = None;
            for (index, _) in self.indices.iter().enumerate() {
                // TODO: detect parallel nodes

                // Create exclusive node
                let node = self.nodes.add(PipelineNode {
                    first: index as u16,
                    count: 1,
                    next: Default::default(),
                });

                // Link previous node or create new stage
                if let Some(previous_node) = previous_node {
                    self.nodes[previous_node].next = node;
                } else {
                    // Update stage first node
                    self.stages.get_mut(stage).unwrap().first_node = node;
                }

                // Next previous node
                previous_node = Some(node);
            }
        }
    }

    pub fn first_node(&self, stage: StageId) -> Option<NodeId> {
        self.stages.get(stage).map(|stage| stage.first_node)
    }

    pub fn next_node(&self, node: NodeId) -> Option<NodeId> {
        let next = self.nodes[node].next;
        if next.is_null() {
            None
        } else {
            Some(next)
        }
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
            first_node: NodeId::null(),
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
}
