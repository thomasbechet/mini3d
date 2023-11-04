use crate::{
    activity::{ActivityError, ActivityId, ActivityManager},
    api::{time::TimeAPI, Context},
    feature::core::resource::ResourceTypeHandle,
    input::InputManager,
    logger::LoggerManager,
    platform::PlatformManager,
    renderer::RendererManager,
    resource::ResourceManager,
    utils::slotmap::{SlotId, SlotMap},
};

use self::{
    container::ContainerTable,
    entity::{EntityChange, EntityEntry, EntityTable},
    query::QueryTable,
    scheduler::Scheduler,
    system::{SystemInstance, SystemTable},
};

pub mod archetype;
pub mod container;
pub mod entity;
pub mod error;
pub mod query;
pub mod scheduler;
pub mod sparse;
pub mod system;
pub mod view;

pub(crate) struct ECSInstance {
    pub(crate) owner: ActivityId,
    pub(crate) containers: ContainerTable,
    pub(crate) entities: EntityTable,
    pub(crate) queries: QueryTable,
    pub(crate) systems: SystemTable,
    pub(crate) scheduler: Scheduler,
    pub(crate) global_cycle: u32,
}

#[derive(Default)]
pub(crate) struct ECSTypes {
    pub(crate) component: ResourceTypeHandle,
    pub(crate) system: ResourceTypeHandle,
    pub(crate) system_stage: ResourceTypeHandle,
    pub(crate) system_set: ResourceTypeHandle,
}

#[derive(Default)]
pub(crate) struct ECSManager {
    entries: SlotMap<ECSInstance>,
    pub(crate) types: ECSTypes,
}

pub(crate) struct ECSUpdateContext<'a> {
    pub(crate) activity: &'a mut ActivityManager,
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) system: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) delta_time: f64,
    pub(crate) global_time: f64,
}

impl ECSManager {
    pub(crate) fn add(&mut self, owner: ActivityId) -> SlotId {
        self.entries.add(ECSInstance {
            owner,
            containers: Default::default(),
            entities: Default::default(),
            queries: Default::default(),
            systems: Default::default(),
            scheduler: Default::default(),
            global_cycle: 0,
        })
    }

    pub(crate) fn remove(&mut self, slot: SlotId) {
        self.entries.remove(slot);
    }

    pub(crate) fn update(&mut self, context: ECSUpdateContext) -> Result<(), ActivityError> {
        // Find active ECS
        let active_ecs = context.activity.entries[context.activity.active.0].ecs;
        let ecs = self.entries.get_mut(active_ecs).unwrap();

        // Begin frame
        ecs.scheduler.begin_frame(context.delta_time);

        // Run stages
        // TODO: protect against infinite loops
        loop {
            // Acquire next node
            let node = ecs.scheduler.next_node();
            if node.is_none() {
                break;
            }
            let node = node.unwrap();

            // Execute node
            if node.count == 1 {
                // Find instance
                let instance = ecs.scheduler.instances[node.first];
                let instance = &ecs.systems.instances[instance];

                // Run the system
                match &instance.instance {
                    SystemInstance::Exclusive(instance) => {
                        let ctx = &mut Context {
                            activity: context.activity,
                            resource: context.resource,
                            input: context.input,
                            renderer: context.renderer,
                            runtime: context.system,
                            logger: context.logger,
                            time: TimeAPI {
                                delta: context.delta_time,
                                global: context.global_time,
                            },
                            containers: &mut ecs.containers,
                            entities: &mut ecs.entities,
                            queries: &mut ecs.queries,
                            scheduler: &mut ecs.scheduler,
                            cycle: ecs.global_cycle,
                        };
                        // TODO: catch unwind
                        instance.run(ctx);
                    }
                    SystemInstance::Parallel(instance) => {
                        let ctx = &Context {
                            activity: context.activity,
                            resource: context.resource,
                            input: context.input,
                            renderer: context.renderer,
                            runtime: context.system,
                            logger: context.logger,
                            time: TimeAPI {
                                delta: context.delta_time,
                                global: context.global_time,
                            },
                            containers: &mut ecs.containers,
                            entities: &mut ecs.entities,
                            queries: &mut ecs.queries,
                            scheduler: &mut ecs.scheduler,
                            cycle: ecs.global_cycle,
                        };
                        // TODO: catch unwind
                        instance.run(ctx);
                    }
                }

                // Flush structural changes
                {
                    // Entity changes
                    for change in ecs.entities.changes.drain(..) {
                        match change {
                            EntityChange::Added(entity) => {
                                // Set default entity archetype
                                let archetype =
                                    &mut ecs.entities.archetypes[ecs.entities.archetypes.empty];
                                let pool_index = archetype.pool.len();
                                archetype.pool.push(entity);
                                // Update entity info
                                ecs.entities.entries.set(
                                    entity.key(),
                                    EntityEntry {
                                        archetype: ecs.entities.archetypes.empty,
                                        pool_index: pool_index as u32,
                                    },
                                );
                            }
                            EntityChange::Removed(entity) => {
                                ecs.entities.remove(entity, &mut ecs.containers);
                            }
                        }
                    }
                    // Component changes
                    for write in instance.writes {
                        let entry = ecs.containers.entries.get_mut(write.0).unwrap();
                        entry
                            .container
                            .flush_changes(&mut ecs.entities, &mut ecs.queries, write);
                    }
                }
            } else {
                // TODO: use thread pool
            }
        }

        Ok(())
    }
}
