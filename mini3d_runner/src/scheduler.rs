use crate::container::ContainerTable;
use crate::ecs::ECS;
use crate::error::ComponentError;
use alloc::{collections::VecDeque, vec::Vec};
use mini3d_derive::Serialize;
use mini3d_utils::slotmap::Key;
use mini3d_utils::string::AsciiArray;
use mini3d_utils::{slot_map_key, slotmap::SlotMap};

pub enum Invocation {
    Immediate,
    NextFrame,
}

slot_map_key!(NodeKey);
slot_map_key!(StageId);
slot_map_key!(SystemId);

#[derive(Default, Serialize)]
pub enum SystemKind {
    Native(#[serialize(skip)] fn()), // In option to allow serialization
    #[default]
    Script,
}

#[derive(Default, Serialize)]
pub struct SystemOrder {}

#[derive(Default, Serialize)]
pub struct System {
    pub(crate) name: AsciiArray<32>,
    pub(crate) stage: StageId,
    pub(crate) order: SystemOrder,
    pub(crate) kind: SystemKind,
}

pub(crate) struct Stage {
    pub(crate) name: AsciiArray<32>,
    pub(crate) first_node: NodeKey,
}

#[derive(Clone, Copy)]
pub(crate) struct PipelineNode {
    pub(crate) first: u16, // Offset in instance indices
    pub(crate) count: u16, // Number of instances
    pub(crate) next: NodeKey,
}

#[derive(Default)]
pub(crate) struct Scheduler<Context> {
    nodes: SlotMap<NodeKey, PipelineNode>,
    pub(crate) callbacks: Vec<fn(&mut ECS<Context>)>,
    next_frame_stages: VecDeque<StageId>,
    frame_stages: VecDeque<StageId>,
    next_node: NodeKey,
    stages: SlotMap<StageId, Stage>,
    systems: SlotMap<SystemId, System>,
    pub(crate) tick_stage: StageId,
}

impl<Context> Scheduler<Context> {
    pub(crate) fn new() -> Self {
        let mut sched = Self {
            nodes: Default::default(),
            callbacks: Default::default(),
            next_frame_stages: Default::default(),
            frame_stages: Default::default(),
            next_node: Default::default(),
            stages: Default::default(),
            systems: Default::default(),
            tick_stage: Default::default(),
        };
        sched.add_stage("tick").unwrap();
        sched
    }

    pub(crate) fn rebuild(&mut self) {
        // Reset baked resources
        self.nodes.clear();
        self.callbacks.clear();
        self.next_node = NodeKey::null();

        // Reset stage entry nodes
        for stage in self.stages.values_mut() {
            stage.first_node = NodeKey::null();
        }

        // Collect stages
        for stage in self.stages.keys() {
            // Collect callbacks
            let callbacks = self
                .systems
                .iter()
                .filter_map(|(_, system)| {
                    if system.stage == stage {
                        match system.kind {
                            SystemKind::Native(callback) => {
                                return Some(callback.unwrap());
                            }
                            SystemKind::Script => {
                                return None;
                            }
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
            for callback in callbacks {
                // TODO: detect parallel nodes

                // Insert instance
                self.callbacks.push(
                    *callback
                        .as_ref()
                        .downcast_ref::<fn(&mut ECS<Context>)>()
                        .unwrap(),
                );

                // Create exclusive node
                let node = self.nodes.add(PipelineNode {
                    first: self.callbacks.len() as u16 - 1,
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

    pub(crate) fn next_node(&mut self, containers: &ContainerTable) -> Option<PipelineNode> {
        // Detect end of current stage
        while self.next_node.is_null() {
            // Find next stage
            if let Some(stage) = self.frame_stages.pop_front() {
                // If the stage exists, find first node
                self.next_node = containers
                    .get::<Stage>(self.stage_id)
                    .unwrap()
                    .get(stage)
                    .map(|stage| stage.first_node)
                    .unwrap_or(NodeKey::null());
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

    pub(crate) fn prepare_next_frame_stages(&mut self) {
        // Collect previous frame stages
        self.frame_stages.clear();
        for stage in self.next_frame_stages.drain(..) {
            self.frame_stages.push_back(stage);
        }
        // Append tick stage
        self.frame_stages.push_back(self.tick_stage);
    }

    pub(crate) fn invoke(&mut self, stage: StageId, invocation: Invocation) {
        match invocation {
            Invocation::Immediate => {
                self.frame_stages.push_front(stage);
            }
            Invocation::NextFrame => {
                self.next_frame_stages.push_back(stage);
            }
        }
    }

    pub(crate) fn find_stage(&self, name: &str) -> Option<StageId> {
        self.stages.iter().find_map(|(id, stage)| {
            if stage.name.as_str() == name {
                Some(id)
            } else {
                None
            }
        })
    }

    pub(crate) fn find_system(&self, name: &str) -> Option<SystemId> {
        self.systems.iter().find_map(|(id, system)| {
            if system.name.as_str() == name {
                Some(id)
            } else {
                None
            }
        })
    }

    pub(crate) fn add_stage(&mut self, name: &str) -> Result<StageId, ComponentError> {
        if self.find_stage(name).is_some() {
            return Err(ComponentError::DuplicatedEntry);
        }
        Ok(self.stages.insert(Stage {
            name: AsciiArray::from(name),
            first_node: NodeKey::null(),
        }))
    }

    pub(crate) fn add_system(
        &mut self,
        name: &str,
        stage: StageId,
        order: SystemOrder,
        kind: SystemKind,
    ) -> Result<SystemId, ComponentError> {
        if self.find_system(name).is_some() {
            return Err(ComponentError::DuplicatedEntry);
        }
        Ok(self.systems.insert(System {
            name: AsciiArray::from(name),
            stage,
            order,
            kind,
        }))
    }
}
