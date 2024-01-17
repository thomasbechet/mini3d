#![no_std]

use core::any::Any;

use component::{
    component_type::ComponentType,
    identifier::Identifier,
    system::System,
    system_stage::{self, SystemStage},
};
use container::ContainerTable;
use entity::EntityTable;
use runner::Runner;
use scheduler::Scheduler;
use system::SystemTable;

#[cfg(test)]
extern crate std;

extern crate alloc;

pub mod bitset;
pub mod component;
pub mod container;
pub mod context;
pub mod entity;
pub mod error;
pub mod query;
pub mod runner;
pub mod scheduler;
pub mod system;
pub mod view;
pub mod world;

pub struct ECS {
    entities: EntityTable,
    containers: ContainerTable,
    scheduler: Scheduler,
    systems: SystemTable,
    runner: Runner,
}

impl ECS {
    pub fn new() -> Self {
        let mut ecs = Self {
            entities: EntityTable::default(),
            containers: ContainerTable::default(),
            scheduler: Scheduler::default(),
            systems: SystemTable::default(),
            runner: Runner::default(),
        };

        // Register base ECS component types
        {
            let entity = ecs.entities.spawn();
            ecs.containers
                .component_type_container()
                .add(entity, ComponentType::native::<System>(true));
            ecs.containers.enable_component_type(entity).unwrap();
        }
        {
            let entity = ecs.entities.spawn();
            ecs.containers
                .component_type_container()
                .add(entity, ComponentType::native::<SystemStage>(true));
            ecs.containers.enable_component_type(entity).unwrap();
        }
        {
            let entity = ecs.entities.spawn();
            ecs.containers
                .component_type_container()
                .add(entity, ComponentType::native::<Identifier>(true));
            ecs.containers.enable_component_type(entity).unwrap();
        }

        // Rebuild scheduler
        ecs.scheduler.rebuild(&mut ecs.systems);

        ecs
    }

    pub fn update(&mut self, user: &mut dyn Any) {
        self.runner.run(
            &mut self.scheduler,
            &mut self.systems,
            &mut self.entities,
            &mut self.containers,
            user,
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut ecs = ECS::new();
        ecs.update(&mut ());
    }
}
