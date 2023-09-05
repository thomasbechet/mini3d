use std::collections::VecDeque;

use crate::{
    feature::component::common::program::Program,
    registry::system::SystemRegistry,
    utils::{
        slotmap::{SecondaryMap, SlotId, SlotMap, SparseSecondaryMap},
        uid::UID,
    },
};

use super::{
    api::{
        asset::{ExclusiveAssetAPI, ParallelAssetAPI},
        ecs::{ExclusiveECS, ParallelECS},
        event::EventAPI,
        input::{ExclusiveInputAPI, ParallelInputAPI},
        registry::{
            ExclusiveComponentRegistryAPI, ExclusiveRegistryAPI, ExclusiveSystemRegistryAPI,
            ParallelComponentRegistryAPI, ParallelRegistryAPI, ParallelSystemRegistryAPI,
        },
        renderer::{ExclusiveRendererAPI, ParallelRendererAPI},
        time::TimeAPI,
        ExclusiveAPI, ParallelAPI,
    },
    archetype::ArchetypeTable,
    component::ComponentTable,
    entity::EntityTable,
    error::ECSError,
    instance::{
        AnyStaticExclusiveSystemInstance, AnyStaticParallelSystemInstance, SystemInstanceEntry,
        SystemResult,
    },
    query::QueryTable,
    ECSUpdateContext,
};

pub enum Invocation {
    Immediate,
    EndFrame,
    NextFrame,
}

pub(crate) enum SystemPipelineNode {
    Exclusive { instance: SlotId, next: SlotId },
    Parallel { first_item: SlotId, next: SlotId },
    ParallelItem { instance: SlotId, next: SlotId },
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

pub(crate) struct PeriodicStage {
    pub(crate) stage: UID,
    pub(crate) frequency: f64,
    pub(crate) accumulator: f64,
}

pub(crate) struct StageEntry {
    first_node: SlotId,
    pub(crate) uid: UID,
}

#[derive(Default)]
pub(crate) struct Scheduler {
    // True when rebuild is required
    out_of_date: bool,
    // Specific update stage (build by engine)
    update_stage: SlotId,
    // Mapping between stage and first node
    stages: SparseSecondaryMap<StageEntry>,
    // Baked nodes
    nodes: SlotMap<SystemPipelineNode>,
    // Periodic invocations
    periodic_stages: Vec<PeriodicStage>,
    // Runtime next stage
    next_frame_stages: VecDeque<SlotId>,
    // Concrete system instances
    instances: SecondaryMap<SystemInstanceEntry>,
    // Incremental cycle
    global_cycle: u32,
}

impl Scheduler {
    pub(crate) fn find_stage(&self, uid: UID) -> Option<SlotId> {
        self.stages
            .iter()
            .find_map(|(id, entry)| if entry.uid == uid { Some(id) } else { None })
    }

    pub(crate) fn rebuild(&mut self, registry: &SystemRegistry) {
        // Reset resources
        self.stages.clear();
        self.nodes.clear();
        self.update_stage = SlotId::null();
        for (id, entry) in registry.stages.iter() {
            // Build pipeline
            let mut previous_node = None;
            while let Some(system) = entry.first_system {
                // TODO: generate parallel pipeline steps
                let node_id = self.nodes.add(SystemPipelineNode::Exclusive {
                    instance: system.into(),
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
                    self.stages.insert(
                        id,
                        StageEntry {
                            first_node: node_id,
                            uid: entry.uid,
                        },
                    );
                }
                // Create instance
                if !self.instances.contains(system.into()) {
                    self.instances
                        .insert(system.into(), SystemInstanceEntry::new(system, registry));
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
        context: &mut ECSUpdateContext,
    ) -> Result<(), ECSError> {
        // Collect previous frame stages
        let mut frame_stages = self.next_frame_stages.drain(..).collect::<VecDeque<_>>();

        // Integrate fixed update stages
        for stage in self.periodic_stages.iter_mut() {
            stage.accumulator += context.delta_time;
            let frequency = stage.frequency;
            let count = (stage.accumulator / frequency) as u32;
            stage.accumulator -= count as f64 * frequency;
            for _ in 0..count {
                frame_stages.push_back(stage.id);
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
                                    manager: context.asset,
                                },
                                input: ExclusiveInputAPI {
                                    manager: context.input,
                                    backend: context.input_backend,
                                },
                                registry: ExclusiveRegistryAPI {
                                    systems: ExclusiveSystemRegistryAPI {
                                        manager: &mut context.registry.systems,
                                    },
                                    components: ExclusiveComponentRegistryAPI {
                                        manager: &mut context.registry.components,
                                    },
                                },
                                renderer: ExclusiveRendererAPI {
                                    manager: context.renderer,
                                    backend: context.renderer_backend,
                                },
                                event: EventAPI {
                                    system: &context.system_backend.events(),
                                },
                                time: TimeAPI {
                                    delta: context.delta_time,
                                    global: context.global_time,
                                },
                            };
                            let ecs = &mut ExclusiveECS {
                                archetypes,
                                components,
                                entities,
                                queries,
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
                                    systems: ParallelSystemRegistryAPI {
                                        manager: &context.registry.systems,
                                    },
                                    components: ParallelComponentRegistryAPI {
                                        manager: &context.registry.components,
                                    },
                                },
                                renderer: ParallelRendererAPI {
                                    manager: context.renderer,
                                },
                                event: EventAPI {
                                    system: &context.system_backend.events(),
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
