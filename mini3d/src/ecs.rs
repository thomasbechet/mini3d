use alloc::vec::Vec;
use mini3d_derive::Error;

use crate::{
    api::{time::TimeAPI, Context},
    input::InputManager,
    logger::LoggerManager,
    math::fixed::{FixedPoint, U32F16},
    platform::PlatformManager,
    renderer::RendererManager,
    resource::{ResourceManager, ResourceTypeHandle},
};

use self::{
    container::ContainerTable,
    entity::{Entity, EntityEntry, EntityTable},
    query::QueryTable,
    resource::{SystemSetHandle, SystemStageHandle},
    scheduler::Scheduler,
    system::{SystemInstance, SystemTable},
};

pub mod archetype;
pub mod component;
pub mod container;
pub mod entity;
pub mod error;
pub mod query;
pub mod resource;
pub mod scheduler;
pub mod sparse;
pub mod system;
pub mod view;

#[derive(Debug, Error)]
pub enum ECSError {
    #[error("progress")]
    Progress,
}

pub enum ECSCommand {
    AddSystemSet(SystemSetHandle),
    RemoveSystemSet(SystemSetHandle),
    SetTargetTPS(u16),
}

#[derive(Default)]
pub(crate) struct ECSHandles {
    pub(crate) component: ResourceTypeHandle,
    pub(crate) system: ResourceTypeHandle,
    pub(crate) system_stage: ResourceTypeHandle,
    pub(crate) system_set: ResourceTypeHandle,
    pub(crate) start_stage: SystemStageHandle,
    pub(crate) tick_stage: SystemStageHandle,
}

#[derive(Default)]
pub(crate) struct ECSManager {
    pub(crate) handles: ECSHandles,
    pub(crate) containers: ContainerTable,
    pub(crate) entities: EntityTable,
    pub(crate) queries: QueryTable,
    pub(crate) scheduler: Scheduler,
    pub(crate) systems: SystemTable,
    pub(crate) entity_created: Vec<Entity>,
    pub(crate) entity_destroyed: Vec<Entity>,
    pub(crate) commands: Vec<ECSCommand>,
    pub(crate) frame_index: u64,
    pub(crate) target_tps: u16,
}

pub(crate) struct ECSUpdateContext<'a> {
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) platform: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
}

impl ECSManager {
    pub(crate) fn flush_commands(&mut self, resource: &mut ResourceManager) {
        for command in self.commands.drain(..).collect::<Vec<_>>() {
            match command {
                ECSCommand::AddSystemSet(set) => {
                    self.systems
                        .insert_system_set(
                            set,
                            &mut self.entities,
                            &mut self.queries,
                            &mut self.containers,
                            resource,
                        )
                        .expect("Failed to insert system set");
                    self.scheduler.rebuild(&self.systems, resource);
                }
                ECSCommand::RemoveSystemSet(set) => todo!(),
                ECSCommand::SetTargetTPS(tps) => {
                    self.target_tps = tps;
                }
            }
        }
    }

    pub(crate) fn flush_changes(&mut self, instance: usize) {
        // Flush structural changes
        let writes = &self.systems.instances[instance].writes;
        {
            // Added entities
            for entity in self.entity_created.drain(..) {
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
            // Component changes
            for write in writes.iter().copied() {
                let entry = self.containers.entries.get_mut(write).unwrap();
                // Component added
                entry.container.get_mut().flush_added_removed(
                    &mut self.entities,
                    &mut self.queries,
                    write,
                );
            }
            // Destroyed entities
            for entity in self.entity_destroyed.drain(..) {
                self.entities.remove(entity, &mut self.containers);
            }
            // Update view sizes
            for write in writes.iter().copied() {
                let entry = self.containers.entries.get_mut(write).unwrap();
                entry.container.get_mut().update_view_size();
            }
        }
    }

    pub(crate) fn update(&mut self, context: ECSUpdateContext) -> Result<(), ECSError> {
        // Find active ECS
        let delta_time = U32F16::ONE / self.target_tps as u32;

        // Begin frame
        self.scheduler
            .invoke_frame_stages(delta_time, self.handles.tick_stage);

        // Run stages
        // TODO: protect against infinite loops
        loop {
            // Acquire next node
            let node = self.scheduler.next_node();
            if node.is_none() {
                break;
            }
            let node = node.unwrap();

            // Execute node
            if node.count == 1 {
                // Find instance
                let instance_index = self.scheduler.instance_indices[node.first];
                let instance = &self.systems.instances[instance_index];

                // Run the system
                match &instance.instance {
                    SystemInstance::Exclusive(instance) => {
                        let ctx = &mut Context {
                            entities: &mut self.entities,
                            entity_created: &mut self.entity_created,
                            entity_destroyed: &mut self.entity_destroyed,
                            scheduler: &mut self.scheduler,
                            resource: context.resource,
                            input: context.input,
                            renderer: context.renderer,
                            platform: context.platform,
                            logger: context.logger,
                            time: TimeAPI {
                                delta: delta_time,
                                frame: self.frame_index,
                                target_tps: self.target_tps,
                            },
                            ecs_types: &self.handles,
                            commands: &mut self.commands,
                        };
                        // TODO: catch unwind
                        instance.run(ctx);
                    }
                    SystemInstance::Parallel(instance) => {
                        let ctx = &Context {
                            entities: &mut self.entities,
                            entity_created: &mut self.entity_created,
                            entity_destroyed: &mut self.entity_destroyed,
                            scheduler: &mut self.scheduler,
                            resource: context.resource,
                            input: context.input,
                            renderer: context.renderer,
                            platform: context.platform,
                            logger: context.logger,
                            time: TimeAPI {
                                delta: delta_time,
                                frame: self.frame_index,
                                target_tps: self.target_tps,
                            },
                            ecs_types: &self.handles,
                            commands: &mut self.commands,
                        };
                        // TODO: catch unwind
                        instance.run(ctx);
                    }
                }

                // Flush structural changes
                self.flush_changes(instance_index);
            } else {
                // TODO: use thread pool
            }
        }

        // Integrate global time
        self.frame_index += 1;

        Ok(())
    }
}
