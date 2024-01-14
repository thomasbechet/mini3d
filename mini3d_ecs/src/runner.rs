use alloc::vec::Vec;

use crate::{
    context::SystemCommand, entity::EntityTable, scheduler::Scheduler, system::SystemTable,
};

#[derive(Default)]
pub(crate) struct Runner {
    commands: Vec<SystemCommand>,
}

impl Runner {
    pub(crate) fn run(
        &mut self,
        scheduler: &mut Scheduler,
        systems: &mut SystemTable,
        entities: &mut EntityTable,
    ) {
        // Run stages
        // TODO: protect against infinite loops
        loop {
            // Acquire next node
            let node = scheduler.next_node(systems);
            if node.is_none() {
                break;
            }
            let node = node.unwrap();

            // Execute node
            if node.count == 1 {
                // Find instance
                let instance_index = scheduler.instance_indices[node.first];
                let instance = &systems.instances[instance_index];

                // Run the system
                match &instance.instance {
                    SystemInstance::Exclusive(instance) => {
                        let ctx = &mut Context {
                            entities: &mut self.entities,
                            entity_created: &mut self.entity_created,
                            entity_destroyed: &mut self.entity_destroyed,
                            scheduler: &mut self.scheduler,
                            input: context.input,
                            renderer: context.renderer,
                            platform: context.platform,
                            logger: context.logger,
                            time: TimeContext {
                                delta: delta_time,
                                frame: self.frame_index,
                                target_tps: self.target_tps,
                            },
                            ecs_types: &self.views,
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
                            input: context.input,
                            renderer: context.renderer,
                            platform: context.platform,
                            logger: context.logger,
                            time: TimeContext {
                                delta: delta_time,
                                frame: self.frame_index,
                                target_tps: self.target_tps,
                            },
                            ecs_types: &self.views,
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
    }
}
