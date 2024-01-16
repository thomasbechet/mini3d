use core::any::Any;

use alloc::vec::Vec;

use crate::{
    container::ContainerTable,
    context::{Context, SystemCommand},
    entity::EntityTable,
    scheduler::Scheduler,
    system::{SystemInstance, SystemTable},
    world::World,
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
        containers: &mut ContainerTable,
        user: &mut dyn Any,
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
                let system_key = scheduler.system_keys[node.first as usize];
                let instance = &systems.systems[system_key].instance;

                let mut ctx = Context {
                    commands: &mut self.commands,
                    user,
                };

                // Run the system
                match &instance {
                    SystemInstance::Exclusive(instance) => {
                        instance.borrow_mut().run(&mut ctx);
                    }
                    SystemInstance::Parallel(instance) => {
                        instance.borrow_mut().run(&ctx);
                    }
                    SystemInstance::Global(instance) => {
                        let mut world = World { entities };
                        instance.borrow_mut().run(&mut ctx, &mut world);
                    }
                }

                // Process commands
                for command in self.commands.drain(..) {
                    match command {
                        SystemCommand::EnableSystem(entity) => {
                            systems.enable_system(entity, containers);
                        }
                        SystemCommand::DisableSystem(entity) => {
                            systems.disable_system(entity, containers);
                        }
                        SystemCommand::EnableSystemStage(entity) => {
                            systems.enable_system_stage(entity, containers);
                        }
                        SystemCommand::DisableSystemStage(entity) => {
                            systems.disable_system_stage(entity, containers);
                        }
                        SystemCommand::Despawn(entity) => {
                            entities.despawn(entity, containers);
                        }
                        SystemCommand::EnableComponentType(entity) => {
                            containers.enable_component_type(entity);
                        }
                        SystemCommand::DisableComponentType(entity) => {
                            containers.disable_component_type(entity);
                        }
                    }
                }
            } else {
                // TODO: use thread pool
            }
        }
    }
}
