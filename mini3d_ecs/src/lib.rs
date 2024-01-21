#![no_std]

use core::any::Any;

use component::{
    component_type::{enable_component_type, ComponentType},
    identifier::Identifier,
    system::{enable_system, IntoNativeExclusiveSystem, System, SystemOrder},
    system_stage::{enable_system_stage, SystemStage},
};
use container::ContainerTable;
use entity::EntityTable;
use instance::InstanceTable;
use runner::Runner;
use scheduler::{Invocation, Scheduler};

#[cfg(test)]
extern crate std;

extern crate alloc;

pub mod bitset;
pub mod component;
pub mod container;
pub mod context;
pub mod entity;
pub mod error;
pub mod instance;
pub mod query;
pub mod runner;
pub mod scheduler;
pub mod view;

pub struct ECS {
    entities: EntityTable,
    containers: ContainerTable,
    scheduler: Scheduler,
    instances: InstanceTable,
    runner: Runner,
}

impl ECS {
    pub fn new<Params>(bootstrap: impl IntoNativeExclusiveSystem<Params>) -> Self {
        let mut ecs = Self {
            entities: EntityTable::default(),
            containers: ContainerTable::default(),
            scheduler: Scheduler::default(),
            instances: InstanceTable::default(),
            runner: Runner::default(),
        };

        // Register base ECS component types

        {
            ecs.entities.system_stage_type = ecs.entities.spawn();
            ecs.containers.component_types_mut().add(
                ecs.entities.system_type,
                ComponentType::native::<System>(true),
            );
            ecs.containers.system_key =
                enable_component_type(ecs.entities.system_type, &mut ecs.containers).unwrap();
        }
        {
            ecs.entities.system_stage_type = ecs.entities.spawn();
            ecs.containers.component_types_mut().add(
                ecs.entities.system_stage_type,
                ComponentType::native::<SystemStage>(true),
            );
            ecs.containers.system_stage_key =
                enable_component_type(ecs.entities.system_stage_type, &mut ecs.containers).unwrap();
        }
        {
            ecs.entities.identifier_type = ecs.entities.spawn();
            ecs.containers.component_types_mut().add(
                ecs.entities.identifier_type,
                ComponentType::native::<Identifier>(true),
            );
            ecs.containers.identifier_key =
                enable_component_type(ecs.entities.identifier_type, &mut ecs.containers).unwrap();
        }

        // Register base stages
        {
            let entity = ecs.entities.spawn();
            ecs.containers
                .system_stages()
                .add(entity, SystemStage::new(true));
            ecs.entities.tick_stage = entity;
            enable_system_stage(entity, &mut ecs.containers).unwrap();
        }
        {
            let entity = ecs.entities.spawn();
            ecs.containers
                .system_stages()
                .add(entity, SystemStage::new(true));
            ecs.entities.bootstrap_stage = entity;
            enable_system_stage(entity, &mut ecs.containers).unwrap();
        }

        // Set identifiers
        {
            let identifiers = ecs.containers.identifiers_mut();
            identifiers.add(ecs.entities.tick_stage, Identifier::new(SystemStage::TICK));
            identifiers.add(
                ecs.entities.bootstrap_stage,
                Identifier::new(SystemStage::START),
            );
        }

        // Register boostrap system
        {
            let entity = ecs.entities.spawn();
            ecs.containers.systems().add(
                entity,
                System::world(
                    bootstrap,
                    ecs.runner.bootstrap_stage,
                    SystemOrder::default(),
                ),
            );
            enable_system(entity, &mut ecs.instances, &mut ecs.containers).unwrap();
            // Invoke bootstrap
            ecs.scheduler
                .invoke(ecs.entities.bootstrap_stage, Invocation::NextFrame);
        }

        // Rebuild scheduler
        ecs.scheduler.rebuild(&mut ecs.containers);

        ecs
    }

    pub fn update(&mut self, user: &mut dyn Any) {
        self.runner.run(
            &mut self.scheduler,
            &mut self.instances,
            &mut self.entities,
            &mut self.containers,
            user,
        );
    }
}

#[cfg(test)]
mod test {
    use std::println;

    use crate::view::NativeSingleMut;

    use super::*;

    #[test]
    fn test() {
        let mut ecs = ECS::new(|ctx, world| {
            println!("Bootstrap");
            let systems = world.find("CTY_System").unwrap();
            let systems = world.view_mut::<NativeSingleMut<System>>(systems).unwrap();
        });
        for _ in 0..10 {
            ecs.update(&mut ());
        }
    }
}
