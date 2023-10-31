use mini3d_derive::Error;

use crate::{
    api::{
        activity::{ActivityCommand, ActivityContext},
        time::TimeAPI,
        Context,
    },
    ecs::{
        container::ContainerTable,
        entity::{EntityChange, EntityEntry, EntityTable},
        query::QueryTable,
        scheduler::Scheduler,
        system::{SystemInstance, SystemTable},
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

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ActivityId(pub(crate) u32);

pub(crate) struct ActivityEntry {
    pub(crate) id: ActivityId,
    pub(crate) name: AsciiArray<32>,
    pub(crate) parent: ActivityId,
    pub(crate) containers: ContainerTable,
    pub(crate) entities: EntityTable,
    pub(crate) queries: QueryTable,
    pub(crate) systems: SystemTable,
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
    pub(crate) root: ActivityId,
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
            parent: Default::default(),
            containers: ContainerTable::new(),
            entities: EntityTable::new(),
            queries: QueryTable::new(),
            systems: SystemTable::new(),
            scheduler: Scheduler::default(),
            global_cycle: 0,
        });
        manager.active = manager.root;
        manager
    }
}

impl ActivityManager {
    fn remove(&mut self, activity: ActivityId) {
        // Find childs
        let childs = self
            .entries
            .values()
            .filter_map(|e| {
                if e.parent == activity {
                    Some(e.id)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        // Remove childs recursively
        for child in childs {
            self.remove(child);
        }
        // Remove activity
        let slot = self
            .entries
            .iter()
            .find_map(|(id, e)| if e.id == activity { Some(id) } else { None })
            .unwrap();
        self.entries.remove(slot);
    }

    pub(crate) fn update(&mut self, context: ActivityUpdateContext) -> Result<(), ActivityError> {
        let activity = self.entries.get_mut(self.active).unwrap();
        self.context.active = activity.id;

        // Begin frame
        activity.scheduler.begin_frame(context.delta_time);

        // Run stages
        // TODO: protect against infinite loops
        loop {
            // Acquire next node
            let node = activity.scheduler.next_node();
            if node.is_none() {
                break;
            }
            let node = node.unwrap();

            // Execute node
            if node.count == 1 {
                // Find instance
                let instance = activity.scheduler.instances[node.first];
                let instance = &activity.systems.instances[instance];

                // Run the system
                match &instance.instance {
                    SystemInstance::Exclusive(instance) => {
                        let ctx = &mut Context {
                            activity: activity.id,
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
                    SystemInstance::Parallel(instance) => {
                        let ctx = &Context {
                            activity: activity.id,
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

                // Flush structural changes
                {
                    // Entity changes
                    for change in activity.entities.changes.drain(..) {
                        match change {
                            EntityChange::Added(entity) => {
                                // Set default entity archetype
                                let archetype = &mut activity.entities.archetypes
                                    [activity.entities.archetypes.empty];
                                let pool_index = archetype.pool.len();
                                archetype.pool.push(entity);
                                // Update entity info
                                activity.entities.entries.set(
                                    entity.key(),
                                    EntityEntry {
                                        archetype,
                                        pool_index: pool_index as u32,
                                    },
                                );
                            }
                            EntityChange::Removed(entity) => {
                                activity.entities.remove(entity, &mut activity.containers);
                            }
                        }
                    }
                    // Component changes
                    for write in instance.writes {
                        let entry = activity.containers.entries.get_mut(write.0).unwrap();
                        entry.container.flush_changes();
                    }
                }
            } else {
                // TODO: use thread pool
            }
        }

        // Process command activities
        for command in self.context.commands.drain(..) {
            match command {
                ActivityCommand::Start(id, descriptor) => {}
                ActivityCommand::Stop(id) => {
                    self.remove(id);
                }
                ActivityCommand::InjectSystemSet(id, system_set) => {}
            }
        }

        Ok(())
    }
}
