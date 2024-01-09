use alloc::{collections::VecDeque, vec::Vec};
use mini3d_derive::fixed;

use crate::{
    math::fixed::U32F16,
    slot_map_key,
    utils::{
        slotmap::{Key, SlotMap},
        uid::UID,
    },
};

use super::{
    component::{ComponentError, SystemStage},
    entity::Entity,
    system::SystemTable,
};

pub enum Invocation {
    Immediate,
    EndFrame,
    NextFrame,
}

slot_map_key!(NodeKey);
slot_map_key!(SystemStageKey);

#[derive(Clone, Copy)]
pub(crate) struct SystemPipelineNode {
    pub(crate) first: usize,
    pub(crate) count: usize,
    next: NodeKey,
}

struct PeriodicStage {
    stage: SystemStageKey,
    period: U32F16,
    accumulator: U32F16,
}

struct StageEntry {
    uid: UID,
    first_node: NodeKey,
}

#[derive(Default)]
pub(crate) struct Scheduler {
    // Mapping between stage and first node
    stages: SlotMap<SystemStageKey, StageEntry>,
    // Baked nodes
    nodes: SlotMap<NodeKey, SystemPipelineNode>,
    // Instances
    pub(crate) instance_indices: Vec<usize>,
    // Periodic invocations
    periodic_stages: Vec<PeriodicStage>,
    // Runtime next frame stage
    next_frame_stages: VecDeque<SystemStageKey>,
    // Runtime stages
    frame_stages: VecDeque<SystemStageKey>,
    // Runtime active node
    next_node: NodeKey,
}

impl Scheduler {
    pub(crate) fn add_system_stage(
        &mut self,
        name: &str,
        entity: Entity,
    ) -> Result<SystemStageKey, ComponentError> {
        let uid = UID::from(name);
        if self.stages.iter().any(|(_, e)| e.uid == uid) {
            return Err(ComponentError::DuplicatedEntry);
        }
        let key = self.stages.add(StageEntry {
            uid,
            first_node: Default::default(),
        });
        // TODO: trigger rebuild of scheduler
        Ok(key)
    }

    pub(crate) fn remove_system_stage(
        &mut self,
        key: SystemStageKey,
    ) -> Result<(), ComponentError> {
        self.stages.remove(key);
        // TODO: trigger rebuild of scheduler
        Ok(())
    }

    pub(crate) fn rebuild(&mut self, table: &SystemTable) {
        // Reset baked resources
        self.stages.clear();
        self.nodes.clear();
        self.instance_indices.clear();
        self.next_node = NodeKey::null();

        // Collect stages
        let mut stages = Vec::new();
        for instance in table.instances.iter() {
            if !stages.iter().any(|(stage, node)| *stage == instance.stage) {
                stages.push(instance.stage);
                let stage = resource.native::<SystemStage>(instance.stage).unwrap();
                if let Some(periodic) = stage.periodic {
                    self.periodic_stages.push(PeriodicStage {
                        stage: instance.stage,
                        period: periodic,
                        accumulator: fixed!(0),
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
                .filter_map(|(index, e)| if e.0 == *stage { Some(index) } else { None })
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
                    self.stages[stage_index].1 = node;
                }

                // Next previous node
                previous_node = Some(node);
            }
        }
    }

    pub(crate) fn invoke_frame_stages(&mut self, delta_time: U32F16, tick_stage: SystemStageKey) {
        // Collect previous frame stages
        self.frame_stages.clear();
        for stage in self.next_frame_stages.drain(..) {
            self.frame_stages.push_back(stage);
        }

        // Integrate fixed update stages
        for stage in self.periodic_stages.iter_mut() {
            stage.accumulator += delta_time;
            let period = stage.period;
            let count = (stage.accumulator / period).int();
            stage.accumulator -= count * period;
            for _ in 0..count {
                self.frame_stages.push_back(stage.stage);
            }
        }

        // Append update stage
        self.frame_stages.push_back(tick_stage);
    }

    pub(crate) fn next_node(&mut self) -> Option<SystemPipelineNode> {
        // Detect end of current stage
        while self.next_node.is_null() {
            // Find next stage
            if let Some(stage) = self.frame_stages.pop_front() {
                // If the stage exists, find first node
                if let Some(index) = self.stages.iter().position(|e| e.0 == stage) {
                    self.next_node = self.stages[index].1;
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

    pub(crate) fn invoke(&mut self, stage: SystemStageKey, invocation: Invocation) {
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
