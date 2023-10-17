use mini3d_derive::Error;

use crate::{
    ecs::{
        api::{activity::ActivityContext, context::Context, time::TimeAPI},
        container::ContainerTable,
        entity::EntityTable,
        query::QueryTable,
        scheduler::Scheduler,
        system::SystemInstance,
    },
    input::InputManager,
    logger::LoggerManager,
    platform::PlatformManager,
    renderer::RendererManager,
    resource::ResourceManager,
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
    },
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ActivityId(pub(crate) u32);

pub(crate) struct ActivityEntry {
    pub(crate) id: ActivityId,
    pub(crate) name: AsciiArray<32>,
    pub(crate) parent: SlotId,
    pub(crate) containers: ContainerTable,
    pub(crate) entities: EntityTable,
    pub(crate) queries: QueryTable,
    pub(crate) instances: SystemInstanceTable,
    pub(crate) scheduler: Scheduler,
    pub(crate) global_cycle: u32,
}

pub(crate) struct ActivityUpdateContext<'a> {
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) system: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) delta_time: f64,
    pub(crate) global_time: f64,
}

#[derive(Debug, Error)]
pub enum ActivityError {
    #[error("progress")]
    Progress,
}

pub(crate) struct ActivityManager {
    pub(crate) root: SlotId,
    pub(crate) active: SlotId,
    pub(crate) entries: SlotMap<ActivityEntry>,
    pub(crate) context: ActivityContext,
}

impl Default for ActivityManager {
    fn default() -> Self {
        let mut manager = Self {
            root: SlotId::null(),
            active: SlotId::null(),
            entries: SlotMap::new(),
            context: Default::default(),
        };
        manager.root = manager.entries.add(ActivityEntry {
            id: ActivityId(0),
            name: AsciiArray::from("root"),
            parent: SlotId::null(),
        });
        manager.active = manager.root;
        manager
    }
}

impl ActivityManager {
    fn add(&mut self, name: &str, parent: SlotId) -> ActivityId {
        if self.entries.values().find(|e| e.name == name).is_some() {
            panic!("Duplicated activity name: {}", name);
        }
        let id = self.next_id;
        self.next_id.0 += 1;
        let entry = ActivityEntry {
            id,
            name: AsciiArray::from(name),
            parent,
            containers: ContainerTable::new(),
            entities: EntityTable::new(),
            queries: QueryTable::new(),
            instances: SystemInstanceTable::new(),
        };
        self.entries.add(entry);
        id
    }

    fn remove(&mut self, activity: ActivityId) {
        let childs = self
            .entries
            .iter()
            .filter(|(_, e)| e.parent == activity.0)
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        for child in childs {
            self.remove(child);
        }
        self.entries.remove(activity.0);
    }

    pub(crate) fn update(&mut self, context: ActivityUpdateContext) -> Result<(), ActivityError> {
        let activity = self.entries.get_mut(self.active).unwrap();

        // Begin frame
        activity.scheduler.begin_frame(context.delta_time);

        // Update cycle
        // Run stages
        // TODO: protect against infinite loops
        loop {
            // Check registry update
            if context.registry.resource.changed {
                context
                    .resource
                    .on_registry_update(&context.registry.resource);
                context.registry.resource.changed = false;
            }
            if context.registry.system.changed || context.registry.component.changed {
                // Update ECS
                self.on_registry_update(context.registry)?;
                context.registry.system.changed = false;
                context.registry.component.changed = false;
            }

            // Acquire next node
            let node = self.scheduler.next_node();
            if node.is_none() {
                break;
            }
            let node = node.unwrap();

            // Execute node
            if node.count == 1 {
                // Find instance
                let instance = self.scheduler.instances[node.first];
                let instance = &self.instances.entries[instance.0];

                // Run the system
                match &instance.system {
                    SystemInstance::Exclusive(instance) => {
                        let ctx = &mut Context {
                            resource: context.resource,
                            input: context.input,
                            renderer: context.renderer,
                            runtime: context.system,
                            logger: context.logger,
                            time: TimeAPI {
                                delta: context.delta_time,
                                global: context.global_time,
                            },
                            containers: &mut self.containers,
                            entities: &mut self.entities,
                            queries: &mut self.queries,
                            scheduler: &mut self.scheduler,
                            cycle: self.global_cycle,
                        };
                        // TODO: catch unwind
                        instance.run(ctx);
                    }
                    SystemInstance::Parallel(instance) => {
                        let ctx = &Context {
                            resource: context.resource,
                            input: context.input,
                            renderer: context.renderer,
                            runtime: context.system,
                            logger: context.logger,
                            time: TimeAPI {
                                delta: context.delta_time,
                                global: context.global_time,
                            },
                            containers: &mut activity.containers,
                            entities: &mut activity.entities,
                            queries: &mut activity.queries,
                            scheduler: &mut activity.scheduler,
                            cycle: activity.global_cycle,
                        };
                        // TODO: catch unwind
                        instance.run(ctx);
                    }
                }
            } else {
                // TODO: use thread pool
            }
        }

        Ok(())
    }
}
