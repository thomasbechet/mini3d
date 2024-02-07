use crate::container::ContainerTable;
use crate::entity::Entity;
use crate::instance::InstanceIndex;
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
    pub(crate) first: u16, // Offset in instance indices
    pub(crate) count: u16, // Number of instances
    pub(crate) next: NodeKey,
}

#[derive(Default)]
pub(crate) struct Scheduler {
    // Baked nodes
    nodes: SlotMap<NodeKey, PipelineNode>,
    // Instances
    pub(crate) instance_indices: Vec<InstanceIndex>,
    // Runtime next frame stage
    next_frame_stages: VecDeque<Entity>,
    // Runtime stages
    frame_stages: VecDeque<Entity>,
    // Runtime active node
    next_node: NodeKey,
}

impl Scheduler {
    pub(crate) fn rebuild(&mut self, containers: &mut ContainerTable) {
        // Reset baked resources
        self.nodes.clear();
        self.instance_indices.clear();
        self.next_node = NodeKey::null();

        // Reset stage entry nodes
        let stages = containers.system_stages();
        for (_, stage) in stages.iter_mut() {
            stage.first_node = NodeKey::null();
        }

        // Collect stages
        let stages = stages.iter().map(|(e, _)| e).collect::<Vec<_>>();
        for stage in stages {
            // Collect instance indices in stage
            let systems = containers.systems();
            let instance_indices = systems
                .iter()
                .filter_map(|(_, data)| {
                    if data.stage == stage {
                        if let Some(instance) = data.instance {
                            return Some(instance);
                        }
                    }
                    None
                })
                .collect::<Vec<_>>();
            // Sort instances based on system order
            // TODO:
            // Create stage entry
            // let stage = resources.get::<SystemStage>(*stage).unwrap();
            // Build nodes
            let mut previous_node = None;
            for instance_index in instance_indices {
                // TODO: detect parallel nodes

                // Insert instance
                self.instance_indices.push(instance_index);

                // Create exclusive node
                let node = self.nodes.add(PipelineNode {
                    first: self.instance_indices.len() as u16 - 1,
                    count: 1,
                    next: Default::default(),
                });

                // Link previous node or create new stage
                if let Some(previous_node) = previous_node {
                    self.nodes[previous_node].next = node;
                } else {
                    // Update stage first node
                    containers
                        .system_stages()
                        .get_mut(stage)
                        .unwrap()
                        .first_node = node;
                }

                // Next previous node
                previous_node = Some(node);
            }
        }
    }

    pub(crate) fn next_node(&mut self, containers: &mut ContainerTable) -> Option<PipelineNode> {
        // Detect end of current stage
        while self.next_node.is_null() {
            // Find next stage
            if let Some(stage) = self.frame_stages.pop_front() {
                // If the stage exists, find first node
                let stages = containers.system_stages();
                // if let Some(stage) = stages.get(stage) {
                // self.next_node = stage.first_node;
                // }
                self.next_node = stages.get(stage).unwrap().first_node;
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

    pub(crate) fn prepare_next_frame_stages(&mut self, tick_stage: Entity) {
        // Collect previous frame stages
        self.frame_stages.clear();
        for stage in self.next_frame_stages.drain(..) {
            self.frame_stages.push_back(stage);
        }
        // Append tick stage
        self.frame_stages.push_back(tick_stage);
    }

    pub(crate) fn invoke(&mut self, stage: Entity, invocation: Invocation) {
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
