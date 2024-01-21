use core::any::Any;

use alloc::vec::Vec;

use crate::{
    component::{
        component_type::{disable_component_type, enable_component_type},
        system::{disable_system, enable_system},
        system_stage::{disable_system_stage, enable_system_stage},
    },
    container::ContainerTable,
    context::{Context, SystemCommand},
    entity::EntityTable,
    instance::{Instance, InstanceTable},
    scheduler::Scheduler,
};

#[derive(Default)]
pub(crate) struct Runner {
    commands: Vec<SystemCommand>,
}

impl Runner {
    pub(crate) fn run(
        &mut self,
        scheduler: &mut Scheduler,
        instances: &mut InstanceTable,
        entities: &mut EntityTable,
        containers: &mut ContainerTable,
        user: &mut dyn Any,
    ) {
        // Prepare frame stages
        scheduler.prepare_next_frame_stages(entities.tick_stage);

        // Run stages
        // TODO: protect against infinite loops
        loop {
            // Acquire next node
            let node = scheduler.next_node(containers);
            if node.is_none() {
                break;
            }
            let node = node.unwrap();

            // Execute node
            if node.count == 1 {
                // Find instance
                let index = scheduler.instance_indices[node.first as usize];
                let instance = &instances.entries[index.0 as usize];

                let mut ctx = Context {
                    commands: &mut self.commands,
                    user,
                    entities,
                };

                // Run the system
                match &instance {
                    Instance::Exclusive(instance) => {
                        instance.borrow_mut().run(&mut ctx);
                    }
                    Instance::Parallel(instance) => {
                        instance.borrow_mut().run(&ctx);
                    }
                }

                // Process commands
                let mut rebuild_scheduler = false;
                for command in self.commands.drain(..) {
                    match command {
                        SystemCommand::EnableSystem(entity) => {
                            enable_system(entity, instances, containers);
                            rebuild_scheduler = true;
                        }
                        SystemCommand::DisableSystem(entity) => {
                            disable_system(entity, instances, containers);
                            rebuild_scheduler = true;
                        }
                        SystemCommand::EnableSystemStage(entity) => {
                            enable_system_stage(entity, containers);
                            rebuild_scheduler = true;
                        }
                        SystemCommand::DisableSystemStage(entity) => {
                            disable_system_stage(entity, containers);
                            rebuild_scheduler = true;
                        }
                        SystemCommand::Despawn(entity) => {
                            entities.despawn(entity, containers);
                        }
                        SystemCommand::EnableComponentType(entity) => {
                            enable_component_type(entity, containers);
                            rebuild_scheduler = true;
                        }
                        SystemCommand::DisableComponentType(entity) => {
                            disable_component_type(entity, containers);
                            rebuild_scheduler = true;
                        }
                        SystemCommand::ReflectEntity(src, dst) => {}
                    }
                }

                // Rebuild scheduler
                if rebuild_scheduler {
                    scheduler.rebuild(containers);
                }
            } else {
                // TODO: use thread pool
            }
        }
    }
}
