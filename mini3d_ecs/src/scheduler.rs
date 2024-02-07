use crate::component::identifier::Identifier;
use crate::component::stage::Stage;
use crate::component::system::{System, SystemKind};
use crate::component::{NamedComponent, RegisterComponent};
use crate::container::{ComponentId, ContainerTable, NativeContainer};
use crate::ecs::ECS;
use crate::entity::Entity;
use alloc::{collections::VecDeque, vec::Vec};
use mini3d_utils::slotmap::Key;
use mini3d_utils::{slot_map_key, slotmap::SlotMap};

pub enum Invocation {
    Immediate,
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
    nodes: SlotMap<NodeKey, PipelineNode>,
    pub(crate) callbacks: Vec<fn(&mut ECS)>,
    next_frame_stages: VecDeque<Entity>,
    frame_stages: VecDeque<Entity>,
    next_node: NodeKey,
    stage_id: ComponentId,
    system_id: ComponentId,
    pub(crate) tick_stage: Entity,
}

impl Scheduler {
    pub(crate) fn setup(ecs: &mut ECS) {
        // Create stage container
        Stage::register(ecs).unwrap();
        ecs.scheduler.stage_id = ecs.find_component_id(Stage::IDENT).unwrap();

        // Create system container
        System::register(ecs).unwrap();
        ecs.scheduler.system_id = ecs.find_component_id(System::IDENT).unwrap();

        // Create tick stage
        let e = ecs.create();
        ecs.add(e, Stage::default());
        ecs.add(e, Identifier::new(Stage::TICK));
        ecs.scheduler.tick_stage = e;
    }

    pub(crate) fn rebuild(&mut self, containers: &mut ContainerTable) {
        // Reset baked resources
        self.nodes.clear();
        self.callbacks.clear();
        self.next_node = NodeKey::null();

        // Reset stage entry nodes
        for (_, stage) in containers
            .get_mut::<Stage>(self.stage_id)
            .unwrap()
            .iter_mut()
        {
            stage.first_node = NodeKey::null();
        }

        // Collect stages
        let stages = containers
            .get::<Stage>(self.stage_id)
            .unwrap()
            .iter()
            .map(|(e, _)| e)
            .collect::<Vec<_>>();
        for stage in stages {
            // Collect callbacks
            let callbacks = containers
                .get::<System>(self.system_id)
                .unwrap()
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
                self.callbacks.push(callback);

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
                    containers
                        .get_mut::<Stage>(self.stage_id)
                        .unwrap()
                        .get_mut(stage)
                        .unwrap()
                        .first_node = node;
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

    pub(crate) fn invoke(&mut self, stage: Entity, invocation: Invocation) {
        match invocation {
            Invocation::Immediate => {
                self.frame_stages.push_front(stage);
            }
            Invocation::NextFrame => {
                self.next_frame_stages.push_back(stage);
            }
        }
    }
}
