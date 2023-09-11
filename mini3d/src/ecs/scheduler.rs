use std::collections::VecDeque;

use crate::{
    registry::system::{System, SystemRegistry, SystemStage},
    utils::{
        slotmap::{SlotId, SlotMap, SparseSecondaryMap},
        uid::UID,
    },
};

use super::error::ECSError;

pub enum Invocation {
    Immediate,
    EndFrame,
    NextFrame,
}

#[derive(Clone, Copy)]
pub(crate) struct SystemPipelineNode {
    pub(crate) first: usize,
    pub(crate) count: usize,
    next: SlotId,
}

struct PeriodicStage {
    stage: UID,
    id: SlotId,
    frequency: f64,
    accumulator: f64,
}

struct StageEntry {
    first_node: SlotId,
    uid: UID,
}

#[derive(Default)]
pub(crate) struct Scheduler {
    // Specific update stage (build by engine)
    update_stage: SlotId,
    // Mapping between stage and first node
    stages: SparseSecondaryMap<StageEntry>,
    // Baked nodes
    nodes: SlotMap<SystemPipelineNode>,
    // Instances
    pub(crate) instances: Vec<System>,
    // Periodic invocations
    periodic_stages: Vec<PeriodicStage>,
    // Runtime next frame stage
    next_frame_stages: VecDeque<SlotId>,
    // Runtime stages
    frame_stages: VecDeque<SlotId>,
    // Runtime active node
    next_node: SlotId,
}

impl Scheduler {
    pub(crate) fn on_registry_update(&mut self, registry: &SystemRegistry) {
        // Reset baked resources
        self.stages.clear();
        self.nodes.clear();
        self.instances.clear();
        self.update_stage = SlotId::null();
        self.next_node = SlotId::null();

        // Reset periodic stages
        for stage in self.periodic_stages.iter_mut() {
            stage.id = SlotId::null();
        }

        // Build nodes from registry stages
        for (id, entry) in registry.stages.iter() {
            // Keep a reference to the update stage
            if entry.uid == SystemStage::UPDATE.into() {
                self.update_stage = id;
            }

            // Add the stage if missing
            if !self.stages.contains(id) {
                self.stages.insert(
                    id,
                    StageEntry {
                        first_node: SlotId::null(),
                        uid: entry.uid,
                    },
                );
            }

            // Build stage nodes
            let mut previous_node = None;
            let mut system = entry.first_system;
            while let Some(instance) = system {
                // TODO: detect parallel nodes

                // Insert instance
                self.instances.push(instance);

                // Create node
                let node = self.nodes.add(SystemPipelineNode {
                    first: self.instances.len() - 1,
                    count: 1,
                    next: SlotId::null(),
                });

                // Iter next system in stage
                system = registry.systems[instance.into()].next_in_stage;

                // Link previous node or create new stage
                if let Some(previous_node) = previous_node {
                    self.nodes[previous_node].next = node;
                } else {
                    // Update stage first node
                    self.stages[id].first_node = node;
                }

                // Next previous node
                previous_node = Some(node);
            }
        }
    }

    pub(crate) fn begin_frame(&mut self, delta_time: f64) {
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
                self.frame_stages.push_back(stage.id);
            }
        }

        // Append update stage
        self.frame_stages.push_back(self.update_stage);
    }

    pub(crate) fn next_node(&mut self) -> Option<SystemPipelineNode> {
        if self.next_node.is_null() {
            if let Some(stage) = self.frame_stages.pop_front() {
                self.next_node = self.stages[stage].first_node;
            } else {
                return None;
            }
        }
        // Find next node
        let node = self.nodes[self.next_node];
        self.next_node = node.next;
        Some(node)
    }

    pub(crate) fn invoke(&mut self, stage: UID, invocation: Invocation) -> Result<(), ECSError> {
        let stage = self
            .stages
            .iter()
            .find_map(
                |(id, entry)| {
                    if entry.uid == stage {
                        Some(id)
                    } else {
                        None
                    }
                },
            )
            .ok_or(ECSError::SystemStageNotFound)?;
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
        Ok(())
    }

    pub(crate) fn set_periodic_invoke(
        &mut self,
        stage: UID,
        frequency: f64,
    ) -> Result<(), ECSError> {
        for periodic_stage in self.periodic_stages.iter_mut() {
            if periodic_stage.stage == stage {
                periodic_stage.frequency = frequency;
                return Ok(());
            }
        }
        self.periodic_stages.push(PeriodicStage {
            stage,
            id: SlotId::null(),
            frequency,
            accumulator: 0.0,
        });
        Ok(())
    }
}
