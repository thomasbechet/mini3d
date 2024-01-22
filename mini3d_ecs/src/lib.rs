// #![no_std]

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
        let mut entities = EntityTable::default();
        let mut containers = ContainerTable::new(entities.spawn());
        let mut scheduler = Scheduler::default();
        let mut instances = InstanceTable::default();
        let mut runner = Runner::default();

        // Register base ECS component types

        {
            entities.system_type = entities.spawn();
            containers
                .component_types_mut()
                .add(entities.system_type, ComponentType::native::<System>(true));
            containers.system_key =
                enable_component_type(entities.system_type, &mut containers).unwrap();
        }
        {
            entities.system_stage_type = entities.spawn();
            containers.component_types_mut().add(
                entities.system_stage_type,
                ComponentType::native::<SystemStage>(true),
            );
            containers.system_stage_key =
                enable_component_type(entities.system_stage_type, &mut containers).unwrap();
        }
        {
            entities.identifier_type = entities.spawn();
            containers.component_types_mut().add(
                entities.identifier_type,
                ComponentType::native::<Identifier>(true),
            );
            containers.identifier_key =
                enable_component_type(entities.identifier_type, &mut containers).unwrap();
        }

        // Register base stages
        {
            let entity = entities.spawn();
            containers
                .system_stages()
                .add(entity, SystemStage::new(true));
            entities.tick_stage = entity;
            enable_system_stage(entity, &mut containers).unwrap();
        }
        {
            let entity = entities.spawn();
            containers
                .system_stages()
                .add(entity, SystemStage::new(true));
            entities.bootstrap_stage = entity;
            enable_system_stage(entity, &mut containers).unwrap();
        }

        // Set identifiers
        {
            let identifiers = containers.identifiers_mut();
            identifiers.add(entities.tick_stage, Identifier::new(SystemStage::TICK));
            identifiers.add(
                entities.bootstrap_stage,
                Identifier::new(SystemStage::START),
            );
        }

        // Register boostrap system
        {
            let entity = entities.spawn();
            containers.systems().add(
                entity,
                System::exclusive(
                    bootstrap,
                    entities.bootstrap_stage,
                    SystemOrder::default(),
                    &[],
                ),
            );
            enable_system(entity, &mut instances, &mut containers).unwrap();
            // Invoke bootstrap
            scheduler.invoke(entities.bootstrap_stage, Invocation::NextFrame);
        }

        // Rebuild scheduler
        scheduler.rebuild(&mut containers);

        Self {
            entities,
            containers,
            scheduler,
            instances,
            runner,
        }
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

    use self::context::Context;

    use super::*;

    fn bootstrap(ctx: &mut Context, cty: NativeSingleMut<ComponentType>) {
        println!("bootstrap");
        for (e, v) in cty.iter() {
            println!("{:?}: {}", e, v.name);
        }
    }

    #[test]
    fn test() {
        let mut ecs = ECS::new(bootstrap);
        for _ in 0..10 {
            ecs.update(&mut ());
        }
    }
}
