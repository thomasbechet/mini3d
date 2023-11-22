use std::collections::VecDeque;

use crate::{
    feature::ecs::system::{SystemStage, SystemStageHandle},
    resource::ResourceManager,
    slot_map_key,
    utils::slotmap::SlotMap,
};

use super::system::SystemTable;

pub enum Invocation {
    Immediate,
    EndFrame,
    NextFrame,
}

slot_map_key!(NodeKey);

#[derive(Clone, Copy)]
pub(crate) struct SystemPipelineNode {
    pub(crate) first: usize,
    pub(crate) count: usize,
    next: NodeKey,
}

struct PeriodicStage {
    stage: SystemStageHandle,
    frequency: f64,
    accumulator: f64,
}

struct StageEntry {
    handle: SystemStageHandle,
    first_node: NodeKey,
}

#[derive(Default)]
pub(crate) struct Scheduler {
    // Mapping between stage and first node
    stages: Vec<StageEntry>,
    // Baked nodes
    nodes: SlotMap<NodeKey, SystemPipelineNode>,
    // Instances
    pub(crate) instance_indices: Vec<usize>,
    // Periodic invocations
    periodic_stages: Vec<PeriodicStage>,
    // Runtime next frame stage
    next_frame_stages: VecDeque<SystemStageHandle>,
    // Runtime stages
    frame_stages: VecDeque<SystemStageHandle>,
    // Runtime active node
    next_node: NodeKey,
}

impl Scheduler {
    pub(crate) fn rebuild(&mut self, table: &SystemTable, resource: &ResourceManager) {
        // Reset baked resources
        self.stages.clear();
        self.nodes.clear();
        self.instance_indices.clear();
        self.next_node = NodeKey::null();

        // Collect stages
        let mut stages = Vec::new();
        for instance in table.instances.iter() {
            if !stages.iter().any(|stage| *stage == instance.stage) {
                stages.push(instance.stage);
                let stage = resource.get::<SystemStage>(instance.stage).unwrap();
                if let Some(periodic) = stage.periodic {
                    self.periodic_stages.push(PeriodicStage {
                        stage: instance.stage,
                        frequency: 1.0 / periodic,
                        accumulator: 0.0,
                    });
                }
            }
        }
        for (stage_index, stage) in stages.iter().enumerate() {
            // Collect instance indices in stage
            let indices = table
                .instances
                .iter()
                .enumerate()
                .filter_map(
                    |(index, e)| {
                        if e.stage == *stage {
                            Some(index)
                        } else {
                            None
                        }
                    },
                )
                .collect::<Vec<_>>();
            // Sort instances based on system order
            // TODO:
            // Create stage entry
            // let stage = resources.get::<SystemStage>(*stage).unwrap();
            self.stages.push(StageEntry {
                handle: *stage,
                first_node: Default::default(),
            });
            // Build nodes
            let mut previous_node = None;
            for index in indices {
                // TODO: detect parallel nodes

                // Insert instance
                self.instance_indices.push(index);

                // Create exclusive node
                let node = self.nodes.add(SystemPipelineNode {
                    first: self.instance_indices.len() - 1,
                    count: 1,
                    next: Default::default(),
                });

                // Link previous node or create new stage
                if let Some(previous_node) = previous_node {
                    self.nodes[previous_node].next = node;
                } else {
                    // Update stage first node
                    self.stages[stage_index].first_node = node;
                }

                // Next previous node
                previous_node = Some(node);
            }
        }
    }

    pub(crate) fn invoke_frame_stages(&mut self, delta_time: f64, update_stage: SystemStageHandle) {
        // Collect previous frame stages
        self.frame_stages.clear();
        for stage in self.next_frame_stages.drain(..) {
            self.frame_stages.push_back(stage);
        }

        // Integrate fixed update stages
        for stage in self.periodic_stages.iter_mut() {
            stage.accumulator += delta_time;
            let frequency = stage.frequency;
            let count = (stage.accumulator / frequency) as u32;
            stage.accumulator -= count as f64 * frequency;
            for _ in 0..count {
                self.frame_stages.push_back(stage.stage);
            }
        }

        // Append update stage
        self.frame_stages.push_back(update_stage);
    }

    pub(crate) fn next_node(&mut self) -> Option<SystemPipelineNode> {
        // Detect end of current stage
        while self.next_node.is_null() {
            // Find next stage
            if let Some(stage) = self.frame_stages.pop_front() {
                // If the stage exists, find first node
                if let Some(index) = self.stages.iter().position(|e| e.handle == stage) {
                    self.next_node = self.stages[index].first_node;
                }
            } else {
                // No more stages
                return None;
            }
        }
        // Find next node
        let node = self.nodes[self.next_node];
        self.next_node = node.next;
        Some(node)
    }

    pub(crate) fn invoke(&mut self, stage: SystemStageHandle, invocation: Invocation) {
        match invocation {
            Invocation::Immediate => {
                self.frame_stages.push_front(stage);
            }
            Invocation::EndFrame => {
                self.frame_stages.push_back(stage);
            }
            Invocation::NextFrame => {
                self.next_frame_stages.push_back(stage);
            }
        }
    }
}
