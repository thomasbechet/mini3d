#![no_std]

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
        ecs
    }

    pub fn update(&mut self) {}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut ecs = ECS::new();
        ecs.update();
    }
}
