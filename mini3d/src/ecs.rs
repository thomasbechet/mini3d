use crate::{
    activity::{ActivityError, ActivityHandle, ActivityManager},
    api::{time::TimeAPI, Context},
    feature::{core::resource::ResourceTypeHandle, ecs::system::SystemStageHandle},
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
    scheduler::{Invocation, Scheduler},
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
    pub(crate) owner: ActivityHandle,
    pub(crate) containers: ContainerTable,
    pub(crate) entities: EntityTable,
    pub(crate) queries: QueryTable,
    pub(crate) systems: SystemTable,
    pub(crate) scheduler: Scheduler,
}

impl ECSInstance {
    pub(crate) fn flush_changes(&mut self, instance: usize) {
        // Flush structural changes
        {
            // Entity changes
            let mut i = 0;
            while i < self.entities.changes.len() {
                let change = self.entities.changes[i];
                match change {
                    EntityChange::Created(entity) => {
                        // Set default entity archetype
                        let archetypes = self.entities.archetypes.get_mut();
                        let archetype = &mut archetypes.entries[archetypes.empty];
                        let pool_index = archetype.pool.len();
                        archetype.pool.push(entity);
                        // Update entity info
                        self.entities.entries.set(
                            entity.key(),
                            EntityEntry {
                                archetype: self.entities.archetypes.get_mut().empty,
                                pool_index: pool_index as u32,
                            },
                        );
                    }
                    EntityChange::Destroyed(entity) => {
                        self.entities.remove(entity, &mut self.containers);
                    }
                }
                i += 1;
            }
            self.entities.changes.clear();
            // Component changes
            for write in &self.systems.instances[instance].writes {
                let entry = self.containers.entries.get_mut(write.0).unwrap();
                entry.container.get_mut().flush_changes(
                    &mut self.entities,
                    &mut self.queries,
                    *write,
                );
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct ECSHandles {
    pub(crate) component: ResourceTypeHandle,
    pub(crate) system: ResourceTypeHandle,
    pub(crate) system_stage: ResourceTypeHandle,
    pub(crate) system_set: ResourceTypeHandle,
    pub(crate) start_stage: SystemStageHandle,
    pub(crate) update_stage: SystemStageHandle,
}

#[derive(Default)]
pub(crate) struct ECSManager {
    pub(crate) instances: SlotMap<ECSInstance>,
    pub(crate) handles: ECSHandles,
}

pub(crate) struct ECSUpdateContext<'a> {
    pub(crate) activity: &'a mut ActivityManager,
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) platform: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) delta_time: f64,
    pub(crate) global_time: f64,
}

impl ECSManager {
    pub(crate) fn add(&mut self, owner: ActivityHandle) -> SlotId {
        let id = self.instances.add(ECSInstance {
            owner,
            containers: Default::default(),
            entities: Default::default(),
            queries: Default::default(),
            systems: Default::default(),
            scheduler: Default::default(),
        });
        self.instances[id]
            .scheduler
            .invoke(self.handles.start_stage, Invocation::NextFrame);
        id
    }

    pub(crate) fn remove(&mut self, slot: SlotId) {
        self.instances.remove(slot);
    }

    pub(crate) fn update(&mut self, context: ECSUpdateContext) -> Result<(), ActivityError> {
        // Find active ECS
        let active_ecs = context.activity.activities[context.activity.active.0].ecs;
        let ecs = self.instances.get_mut(active_ecs).unwrap();

        // Begin frame
        ecs.scheduler
            .invoke_frame_stages(context.delta_time, self.handles.update_stage);

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
                let instance_index = ecs.scheduler.instance_indices[node.first];
                let instance = &ecs.systems.instances[instance_index];

                // Run the system
                match &instance.instance {
                    SystemInstance::Exclusive(instance) => {
                        let ctx = &mut Context {
                            activity: context.activity,
                            resource: context.resource,
                            input: context.input,
                            renderer: context.renderer,
                            platform: context.platform,
                            logger: context.logger,
                            time: TimeAPI {
                                delta: context.delta_time,
                                global: context.global_time,
                            },
                            containers: &mut ecs.containers,
                            entities: &mut ecs.entities,
                            queries: &mut ecs.queries,
                            scheduler: &mut ecs.scheduler,
                            ecs_types: &self.handles,
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
                            platform: context.platform,
                            logger: context.logger,
                            time: TimeAPI {
                                delta: context.delta_time,
                                global: context.global_time,
                            },
                            containers: &mut ecs.containers,
                            entities: &mut ecs.entities,
                            queries: &mut ecs.queries,
                            scheduler: &mut ecs.scheduler,
                            ecs_types: &self.handles,
                        };
                        // TODO: catch unwind
                        instance.run(ctx);
                    }
                }

                // Flush structural changes
                ecs.flush_changes(instance_index);
            } else {
                // TODO: use thread pool
            }
        }

        Ok(())
    }
}
