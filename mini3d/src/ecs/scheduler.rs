use std::collections::VecDeque;

use crate::{
    feature::core::system_stage::SystemStage,
    utils::{
        slotmap::{SlotId, SlotMap},
        uid::{ToUID, UID},
    },
};

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
    frequency: f64,
    accumulator: f64,
}

struct StageEntry {
    first_node: SlotId,
    uid: UID,
}

#[derive(Default)]
pub(crate) struct Scheduler {
    // Mapping between stage and first node
    stages: Vec<StageEntry>,
    // Baked nodes
    nodes: SlotMap<SystemPipelineNode>,
    // Instances
    pub(crate) instances: Vec<System>,
    // Periodic invocations
    periodic_stages: Vec<PeriodicStage>,
    // Runtime next frame stage
    next_frame_stages: VecDeque<UID>,
    // Runtime stages
    frame_stages: VecDeque<UID>,
    // Runtime active node
    next_node: SlotId,
}

impl Scheduler {
    pub(crate) fn log(&self, registry: &SystemRegistryManager) {
        println!("Scheduler:");
        for stage in self.stages.iter() {
            println!("  Stage: {}", stage.uid);
            let mut next = stage.first_node;
            while !next.is_null() {
                let node = self.nodes[next];
                println!("    Node: {} instances", node.count);
                for i in node.first..node.first + node.count {
                    let name = registry.get(self.instances[i]).unwrap().name.as_str();
                    println!("    Instance: {}", name);
                }
                next = node.next;
            }
        }
    }

    pub(crate) fn on_registry_update(&mut self, registry: &SystemRegistryManager) {
        // Reset baked resources
        self.stages.clear();
        self.nodes.clear();
        self.instances.clear();
        self.next_node = SlotId::null();

        // Build nodes from registry stages
        for entry in registry.stages.values() {
            // Find stage index
            let mut stage_index = self.stages.iter().position(|e| e.uid == entry.uid);
            if stage_index.is_none() {
                self.stages.push(StageEntry {
                    first_node: SlotId::null(),
                    uid: entry.uid,
                });
                stage_index = Some(self.stages.len() - 1);
            }
            let stage_index = stage_index.unwrap();

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
                system = registry.systems[instance.0].next_in_stage;

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
                self.frame_stages.push_back(stage.stage);
            }
        }

        // Append update stage
        self.frame_stages.push_back(SystemStage::UPDATE.into());
    }

    pub(crate) fn next_node(&mut self) -> Option<SystemPipelineNode> {
        // Detect end of current stage
        while self.next_node.is_null() {
            // Find next stage
            if let Some(stage) = self.frame_stages.pop_front() {
                // If the stage exists, find first node
                if let Some(index) = self.stages.iter().position(|e| e.uid == stage) {
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

    pub(crate) fn invoke(&mut self, stage: impl ToUID, invocation: Invocation) {
        match invocation {
            Invocation::Immediate => {
                self.frame_stages.push_front(stage.to_uid());
            }
            Invocation::EndFrame => {
                self.frame_stages.push_back(stage.to_uid());
            }
            Invocation::NextFrame => {
                self.next_frame_stages.push_back(stage.to_uid());
            }
        }
        Ok(())
    }

    pub(crate) fn set_periodic_invoke(&mut self, stage: impl ToUID, frequency: f64) {
        let stage = stage.to_uid();
        for periodic_stage in self.periodic_stages.iter_mut() {
            if periodic_stage.stage == stage {
                periodic_stage.frequency = frequency;
                return;
            }
        }
        self.periodic_stages.push(PeriodicStage {
            stage,
            frequency,
            accumulator: 0.0,
        });
    }
}
