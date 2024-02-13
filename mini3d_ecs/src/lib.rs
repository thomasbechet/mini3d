#![no_std]

use container::ContainerTable;
use ecs::ECS;
use registry::Registry;
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
pub mod registry;
pub mod scheduler;

pub struct ECSInstance<Context> {
    containers: ContainerTable,
    registry: Registry,
    scheduler: Scheduler<Context>,
}

impl<Context: Default> ECSInstance<Context> {
    pub fn new(bootstrap: fn(&mut ECS<Context>), context: &mut Context) -> Self {
        let mut instance = Self {
            containers: Default::default(),
            registry: Default::default(),
            scheduler: Default::default(),
        };
        let mut ecs = ECS {
            containers: &mut instance.containers,
            registry: &mut instance.registry,
            scheduler: &mut instance.scheduler,
        };

        // Setup component and identifier containers
        ContainerTable::setup(&mut ecs);
        Scheduler::setup(&mut ecs);

        // Bootstrap
        bootstrap(&mut ecs);

        instance
    }

    pub fn update(&mut self, context: &mut Context) {
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
                    containers: &mut self.containers,
                    registry: &mut self.registry,
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
        component::{
            component::Component, identifier::Identifier, stage::Stage, system::System,
            NamedComponent, RegisterComponent,
        },
        ecs::ECS,
        query::Query,
        scheduler::Invocation,
        ECSInstance,
    };

    use crate as mini3d_ecs2;

    #[derive(Default)]
    struct MyContext;

    #[derive(Default, Serialize, Component)]
    struct MyComponent {
        value: u32,
    }

    fn system1(ecs: &mut ECS<MyContext>) {
        println!("system1");
    }

    fn system2(ecs: &mut ECS<MyContext>) {
        let e = ecs.find("test").unwrap();
        ecs.get_mut::<MyComponent>(e).unwrap().value += 1;
        println!("value: {}", ecs.get::<MyComponent>(e).unwrap().value);
    }

    fn bootstrap(ecs: &mut ECS<MyContext>) {
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
        ecs.add(e, Stage::default());
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

        let q = Query::default().all(&[
            ecs.find_component_id(Component::IDENT).unwrap(),
            ecs.find_component_id(Identifier::IDENT).unwrap(),
            ecs.find_component_id(MyComponent::IDENT).unwrap(),
        ]);
        for e in q.entities(ecs) {
            println!("components: {}", e);
        }

        ecs.for_each2::<Component, Identifier>(|e, c, i| {
            println!("[{}]: {}", e, i.ident());
        });

        ecs.for_each3::<Stage, MyComponent, Identifier>(|e, s, c, i| {
            println!("hell [{}]: {}", e, i.ident());
        });
    }

    #[test]
    fn test() {
        let mut ecs = ECSInstance::new(bootstrap, &mut MyContext);
        for _ in 0..10 {
            ecs.update(&mut MyContext);
        }
    }
}
