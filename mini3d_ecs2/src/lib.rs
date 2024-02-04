#![no_std]

use core::any::Any;

use container::ContainerTable;
use ecs::ECS;
use entity::EntityTable;
use scheduler::Scheduler;

#[cfg(test)]
extern crate std;

extern crate alloc;

pub mod bitset;
pub mod component;
pub mod container;
pub mod ecs;
pub mod entity;
pub mod error;
pub mod query;
pub mod scheduler;

pub struct ECSInstance {
    containers: ContainerTable,
    entities: EntityTable,
    scheduler: Scheduler,
}

impl ECSInstance {
    pub fn new(bootstrap: fn(&mut ECS), user: &mut dyn Any) -> Self {
        let mut instance = Self {
            containers: Default::default(),
            entities: Default::default(),
            scheduler: Default::default(),
        };
        let mut ecs = ECS {
            user,
            entities: &mut instance.entities,
            containers: &mut instance.containers,
            scheduler: &mut instance.scheduler,
        };

        // Setup component and identifier containers
        ContainerTable::setup(&mut ecs);
        Scheduler::setup(&mut ecs);

        // Bootstrap
        bootstrap(&mut ecs);

        instance
    }

    pub fn update(&mut self, user: &mut dyn Any) {
        // Prepare frame stages
        self.scheduler.prepare_next_frame_stages();

        // Run stages
        // TODO: protect against infinite loops
        loop {
            // Acquire next node
            let node = self.scheduler.next_node(&self.containers);
            if node.is_none() {
                break;
            }
            let node = node.unwrap();

            // Execute node
            if node.count == 1 {
                // Find callback
                let callback = self.scheduler.callbacks[node.first as usize];

                // Run the callback
                callback(&mut ECS {
                    user,
                    entities: &mut self.entities,
                    containers: &mut self.containers,
                    scheduler: &mut self.scheduler,
                });
            } else {
                // TODO: use thread pool
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::println;

    use mini3d_derive::{Component, Serialize};

    use crate::{
        component::{identifier::Identifier, stage::Stage, system::System, RegisterComponent},
        ecs::ECS,
        scheduler::Invocation,
        ECSInstance,
    };

    use crate as mini3d_ecs2;

    #[derive(Default, Serialize, Component)]
    struct MyComponent {
        value: u32,
    }

    fn system1(ecs: &mut ECS) {
        println!("system1");
    }

    fn system2(ecs: &mut ECS) {
        let e = ecs.find("test").unwrap();
        ecs.get_mut::<MyComponent>(e).unwrap().value += 1;
        println!("value: {}", ecs.get::<MyComponent>(e).unwrap().value);
    }

    fn bootstrap(ecs: &mut ECS) {
        println!("hello");
        let stage = ecs.create();
        ecs.add(stage, Identifier::new("custom_stage"));
        ecs.add(stage, Stage::default());
        let e = ecs.create();
        ecs.add(e, Identifier::new("system1"));
        ecs.add(e, System::native(system1, stage, Default::default()));
        let e = ecs.create();
        ecs.add(
            e,
            System::native(system2, ecs.tick_stage(), Default::default()),
        );
        MyComponent::register(ecs).unwrap();
        ecs.add(e, MyComponent::default());
        ecs.add(e, Identifier::new("test"));
        ecs.invoke(stage, Invocation::NextFrame);

        let e = ecs.create();
        ecs.destroy(e);
        ecs.create();

        for e in ecs.entities() {
            if ecs.has::<Identifier>(e) {
                println!("[{}] {}", e, ecs.get::<Identifier>(e).unwrap().ident());
            } else {
                println!("[{}] ---", e);
            }
        }
    }

    #[test]
    fn test() {
        let mut ecs = ECSInstance::new(bootstrap, &mut ());
        for _ in 0..10 {
            ecs.update(&mut ());
        }
    }
}
