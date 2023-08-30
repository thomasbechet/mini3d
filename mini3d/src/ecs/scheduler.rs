use std::collections::VecDeque;

use crate::{
    feature::component::common::program::Program,
    registry::system::SystemRegistry,
    utils::slotmap::{SecondaryMap, SlotId, SlotMap, SparseSecondaryMap},
};

use super::{
    api::{
        asset::{ExclusiveAssetAPI, ParallelAssetAPI},
        ecs::{ExclusiveECS, ParallelECS},
        input::{ExclusiveInputAPI, ParallelInputAPI},
        registry::{ExclusiveRegistryAPI, ParallelRegistryAPI},
        renderer::{ExclusiveRendererAPI, ParallelRendererAPI},
        time::TimeAPI,
        ExclusiveAPI, ParallelAPI,
    },
    archetype::ArchetypeTable,
    component::ComponentTable,
    entity::EntityTable,
    error::SceneError,
    query::QueryTable,
    system::{
        AnyStaticExclusiveSystemInstance, AnyStaticParallelSystemInstance, SystemInstanceId,
        SystemResult, SystemStageId, SystemStageKind, SystemTable,
    },
    ECSUpdateContext,
};

pub enum Invocation {
    Immediate,
    EndFrame,
    NextFrame,
}

pub(crate) type SystemPipelineId = SlotId;
pub(crate) type SystemPipelineNodeId = SlotId;

pub(crate) enum SystemPipelineNode {
    Exclusive {
        instance: SystemInstanceId,
        next: SystemPipelineNodeId,
    },
    Parallel {
        first_item: SystemPipelineNodeId,
        next: SystemPipelineNodeId,
    },
    ParallelItem {
        instance: SystemInstanceId,
        next: SystemPipelineNodeId,
    },
}

#[derive(Default)]
pub(crate) struct PipelineSystemStage {
    first_node: SystemPipelineNodeId,
    frequency: Option<f64>,
    accumulator: f64,
}

pub(crate) enum StaticSystemInstance {
    Exclusive(Box<dyn AnyStaticExclusiveSystemInstance>),
    Parallel(Box<dyn AnyStaticParallelSystemInstance>),
}

pub(crate) struct ProgramSystemInstance {
    program: Program,
}

pub(crate) enum SystemInstance {
    Static(StaticSystemInstance),
    Program(ProgramSystemInstance),
}

impl SystemInstance {
    fn run_exclusive(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult {
        match self {
            Self::Static(instance) => match instance {
                StaticSystemInstance::Exclusive(instance) => instance.run(ecs, api),
                StaticSystemInstance::Parallel(_) => unreachable!(),
            },
            Self::Program(_) => Ok(()),
        }
    }
    fn run_parallel(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) -> SystemResult {
        match self {
            Self::Static(instance) => match instance {
                StaticSystemInstance::Parallel(instance) => instance.run(ecs, api),
                StaticSystemInstance::Exclusive(_) => unreachable!(),
            },
            Self::Program(instance) => Ok(()),
        }
    }
}

#[derive(Default)]
pub(crate) struct Scheduler {
    out_of_date: bool,
    // Stages
    stages: SparseSecondaryMap<PipelineSystemStage>,
    update_stage: SystemStageId,
    fixed_update_stages: Vec<SlotId>,
    // Baked resource
    nodes: SlotMap<SystemPipelineNode>,
    instances: SecondaryMap<SystemInstance>,
    // Runtime state
    next_frame_stages: VecDeque<SystemStageId>,
    global_cycle: u32,
}

impl Scheduler {
    pub(crate) fn build(&mut self, systems: &SystemTable, registry: &SystemRegistry) {
        self.nodes.clear();
        self.stages.clear();
        self.update_stage = SystemStageId::null();
        self.fixed_update_stages.clear();
        for (id, entry) in systems.stages.iter() {
            // Build pipeline
            let mut previous_node = None;
            while let Some(instance) = entry.first_instance {
                // TODO: generate parallel pipeline steps
                let node_id = self.nodes.add(SystemPipelineNode::Exclusive {
                    instance,
                    next: SlotId::null(),
                });
                if let Some(previous_node) = previous_node {
                    match &mut self.nodes[previous_node] {
                        SystemPipelineNode::Exclusive { next, .. } => {
                            *next = node_id;
                        }
                        SystemPipelineNode::Parallel { next, .. } => {
                            *next = node_id;
                        }
                        SystemPipelineNode::ParallelItem { next, .. } => {
                            *next = node_id;
                        }
                    }
                } else {
                    // Record baked stage
                    match entry.kind {
                        SystemStageKind::Update => {
                            self.update_stage = id;
                            self.stages.insert(
                                id,
                                PipelineSystemStage {
                                    first_node: node_id,
                                    frequency: None,
                                    accumulator: 0.0,
                                },
                            );
                        }
                        SystemStageKind::FixedUpdate(frequency) => {
                            self.fixed_update_stages.push(id);
                            self.stages.insert(
                                id,
                                PipelineSystemStage {
                                    first_node: node_id,
                                    frequency: Some(frequency),
                                    accumulator: 0.0,
                                },
                            );
                        }
                        SystemStageKind::Event(_) => todo!(),
                    }
                }
                // Create instance
                if !self.instances.contains(instance) {
                    self.instances.insert(
                        instance,
                        registry
                            .get(systems.instances[instance].system)
                            .expect("System not found")
                            .reflection
                            .create_instance(),
                    );
                }
                // Next previous node
                previous_node = Some(node_id);
            }
        }
        self.out_of_date = true;
    }

    pub(crate) fn update(
        &mut self,
        archetypes: &mut ArchetypeTable,
        components: &mut ComponentTable,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        systems: &mut SystemTable,
        context: &mut ECSUpdateContext,
    ) -> Result<(), SceneError> {
        // Collect previous frame stages
        let mut frame_stages = self.next_frame_stages.drain(..).collect::<VecDeque<_>>();

        // Integrate fixed update stages
        for id in self.fixed_update_stages.iter() {
            let stage = self.stages.get_mut(*id).unwrap();
            stage.accumulator += context.delta_time;
            let frequency = stage.frequency.unwrap();
            let count = (stage.accumulator / frequency) as u32;
            stage.accumulator -= count as f64 * frequency;
            for _ in 0..count {
                frame_stages.push_back(*id);
            }
        }

        // Append update stage
        frame_stages.push_back(self.update_stage);

        // Run stages
        // TODO: protect against infinite loops
        while let Some(stage) = frame_stages.pop_front() {
            if let Some(stage) = self.stages.get(stage) {
                let mut current = stage.first_node;
                while !current.is_null() {
                    match self.nodes[current] {
                        SystemPipelineNode::Exclusive { instance, next } => {
                            let api = &mut ExclusiveAPI {
                                asset: ExclusiveAssetAPI {
                                    registry: &context.registry.borrow().components,
                                    manager: context.asset,
                                },
                                input: ExclusiveInputAPI {
                                    manager: context.input,
                                    backend: context.input_backend,
                                },
                                registry: ExclusiveRegistryAPI {
                                    manager: context.registry.get_mut(),
                                },
                                renderer: ExclusiveRendererAPI {
                                    manager: context.renderer,
                                    backend: context.renderer_backend,
                                },
                                time: TimeAPI {
                                    delta: context.delta_time,
                                    fixed: stage.frequency,
                                    global: context.global_time,
                                },
                            };
                            let ecs = &mut ExclusiveECS {
                                archetypes,
                                components,
                                entities,
                                queries,
                                systems,
                                frame_stages: &mut frame_stages,
                                next_frame_stages: &mut self.next_frame_stages,
                                cycle: self.global_cycle,
                            };
                            self.instances[instance].run_exclusive(ecs, api);
                            current = next;
                        }
                        SystemPipelineNode::Parallel { first_item, next } => {
                            // TODO: use thread pool
                            let api = &mut ParallelAPI {
                                asset: ParallelAssetAPI {
                                    manager: context.asset,
                                },
                                input: ParallelInputAPI {
                                    manager: context.input,
                                },
                                registry: ParallelRegistryAPI {
                                    manager: &context.registry.borrow(),
                                },
                                renderer: ParallelRendererAPI {
                                    manager: context.renderer,
                                },
                                time: TimeAPI {
                                    delta: context.delta_time,
                                    fixed: stage.frequency,
                                    global: context.global_time,
                                },
                            };
                            let ecs = &mut ParallelECS {
                                components,
                                entities,
                                queries,
                                cycle: self.global_cycle,
                            };
                            // TODO:
                            current = next;
                        }
                        _ => unreachable!(),
                    }
                }
            }
        }

        Ok(())
    }

    pub(crate) fn cycle(&self) -> u32 {
        self.global_cycle
    }
}
