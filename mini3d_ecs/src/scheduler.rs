use crate::context::SystemCommand;
use crate::system::{SystemKey, SystemStageKey, SystemTable};
use alloc::{collections::VecDeque, vec::Vec};
use mini3d_utils::slotmap::Key;
use mini3d_utils::{slot_map_key, slotmap::SlotMap};

pub enum Invocation {
    Immediate,
    EndFrame,
    NextFrame,
}

slot_map_key!(NodeKey);

#[derive(Clone, Copy)]
pub(crate) struct PipelineNode {
    pub(crate) first: u16,
    pub(crate) count: u16,
    pub(crate) next: NodeKey,
}

#[derive(Default)]
pub(crate) struct Scheduler {
    // Baked nodes
    nodes: SlotMap<NodeKey, PipelineNode>,
    // Instances
    pub(crate) system_keys: Vec<SystemKey>,
    // Runtime next frame stage
    next_frame_stages: VecDeque<SystemStageKey>,
    // Runtime stages
    frame_stages: VecDeque<SystemStageKey>,
    // Runtime active node
    next_node: NodeKey,
    // System command buffer
    commands: Vec<SystemCommand>,
}

impl Scheduler {
    pub(crate) fn rebuild(&mut self, table: &mut SystemTable) {
        // Reset baked resources
        self.nodes.clear();
        self.system_keys.clear();
        self.next_node = NodeKey::null();

        // Reset stage entry nodes
        for stage in table.stages.values_mut() {
            stage.first_node = NodeKey::null();
        }

        // Collect stages
        for (stage_key, stage) in table.stages.iter_mut() {
            // Collect systems in stage
            let system_keys = table
                .systems
                .iter()
                .filter_map(|(key, e)| {
                    if e.stage == stage_key {
                        Some(key)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            // Sort instances based on system order
            // TODO:
            // Create stage entry
            // let stage = resources.get::<SystemStage>(*stage).unwrap();
            // Build nodes
            let mut previous_node = None;
            for system_key in system_keys {
                // TODO: detect parallel nodes

                // Insert instance
                self.system_keys.push(system_key);

                // Create exclusive node
                let node = self.nodes.add(PipelineNode {
                    first: self.system_keys.len() as u16 - 1,
                    count: 1,
                    next: Default::default(),
                });

                // Link previous node or create new stage
                if let Some(previous_node) = previous_node {
                    self.nodes[previous_node].next = node;
                } else {
                    // Update stage first node
                    stage.first_node = node;
                }

                // Next previous node
                previous_node = Some(node);
            }
        }
    }

    pub(crate) fn next_node(&mut self, table: &SystemTable) -> Option<PipelineNode> {
        // Detect end of current stage
        while self.next_node.is_null() {
            // Find next stage
            if let Some(stage) = self.frame_stages.pop_front() {
                // If the stage exists, find first node
                self.next_node = table.stages[stage].first_node;
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
