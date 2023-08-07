use std::collections::VecDeque;

use crate::utils::slotmap::SlotId;

use super::{
    archetype::ArchetypeTable,
    component::ComponentTable,
    context::{
        asset::ExclusiveAssetContext, event::EventContext, input::ExclusiveInputContext,
        registry::RegistryContext, renderer::ExclusiveRendererContext,
        scene::ExclusiveSceneContext, stage::ExclusiveStageContext, time::TimeContext,
        ExclusiveContext, ParallelContext,
    },
    entity::EntityTable,
    error::SceneError,
    query::QueryTable,
    system::{StaticSystemInstance, SystemInstance, SystemStageId, SystemStageKind, SystemTable},
    ECSUpdateContext,
};

pub enum Invocation {
    Immediate,
    EndFrame,
    NextFrame,
}

type FixedStageId = SlotId;

struct FixedStageEntry {
    accumulator: f64,
    stage: SystemStageId,
}

#[derive(Default)]
pub(crate) struct Scheduler {
    update_stage: Option<SystemStageId>,
    fixed_update_stages: Vec<FixedStageEntry>,
    next_frame_stages: VecDeque<SystemStageId>,
    global_cycle: u32,
}

impl Scheduler {
    pub(crate) fn build(&mut self, systems: &SystemTable) {
        self.update_stage = None;
        self.fixed_update_stages.clear();
        for (id, entry) in systems.stages.iter() {
            match entry.stage.kind {
                SystemStageKind::Update => {
                    self.update_stage = Some(id);
                }
                SystemStageKind::FixedUpdate(_) => {
                    self.fixed_update_stages.push(FixedStageEntry {
                        accumulator: 0.0,
                        stage: id,
                    });
                }
                SystemStageKind::Event(_) => todo!(),
            }
        }
    }

    pub(crate) fn update(
        &mut self,
        delta_time: f64,
        global_time: f64,
        archetypes: &mut ArchetypeTable,
        components: &mut ComponentTable,
        entities: &mut EntityTable,
        queries: &mut QueryTable,
        systems: &mut SystemTable,
        mut context: ECSUpdateContext,
    ) -> Result<(), SceneError> {
        // Collect previous frame stages
        let mut frame_stages = self.next_frame_stages.drain(..).collect::<VecDeque<_>>();

        // Integrate fixed update stages
        for stage in self.fixed_update_stages.iter_mut() {
            stage.accumulator += delta_time;
            let frequency = systems.stages[stage.stage].stage.frequency().unwrap();
            let count = (stage.accumulator / frequency) as u32;
            stage.accumulator -= count as f64 * frequency;
            for i in 0..count {
                frame_stages.push_back(stage.stage);
            }
        }

        // Append update stage
        if let Some(stage) = self.update_stage {
            frame_stages.push_back(stage);
        }

        // Run stages
        // TODO: protect against infinite loops
        while let Some(stage) = frame_stages.pop_front() {
            // TODO: precompute pipeline of systems on system table update
            let fixed_delta_time = systems.stages[stage].stage.frequency();
            let mut current = systems.stages[stage].first_instance;
            while let Some(instance) = current {
                if systems.instances[instance].active {
                    match systems.instances[instance].instance {
                        SystemInstance::Static(system) => match system {
                            StaticSystemInstance::Exclusive(system) => {
                                // Run exclusive system
                                system.run(&mut ExclusiveContext {
                                    asset: ExclusiveAssetContext {
                                        registry: &context.registry.borrow().components,
                                        manager: context.asset,
                                    },
                                    event: EventContext {
                                        events: context.events,
                                    },
                                    input: ExclusiveInputContext {
                                        manager: context.input,
                                    },
                                    stage: ExclusiveStageContext {
                                        active_stage: stage,
                                        frame_stages: &mut frame_stages,
                                        next_frame_stages: &mut self.next_frame_stages,
                                    },
                                    registry: RegistryContext {
                                        manager: &context.registry.borrow(),
                                    },
                                    renderer: ExclusiveRendererContext {
                                        manager: context.renderer,
                                    },
                                    scene: ExclusiveSceneContext {
                                        registry: &context.registry.borrow(),
                                        archetypes,
                                        components,
                                        entities,
                                        queries,
                                        cycle: self.global_cycle,
                                    },
                                    time: TimeContext {
                                        delta: delta_time,
                                        fixed: fixed_delta_time,
                                        global: global_time,
                                    },
                                });
                            }
                            StaticSystemInstance::Parallel(system) => {
                                // Run parallel system (TODO: use thread pool)
                                system.run(&mut ParallelContext {
                                    asset: todo!(),
                                    event: todo!(),
                                    input: todo!(),
                                    stage: todo!(),
                                    registry: todo!(),
                                    renderer: todo!(),
                                    scene: todo!(),
                                    scheduler: todo!(),
                                    time: todo!(),
                                });
                            }
                        },
                        SystemInstance::Program(_) => {
                            unimplemented!()
                        }
                    }
                }
                current = systems.instances[instance].next_instance;
            }
        }

        Ok(())
    }

    pub(crate) fn cycle(&self) -> u32 {
        self.global_cycle
    }
}
