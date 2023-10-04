use crate::{
    logger::LoggerManager,
    platform::PlatformManager,
    registry::error::RegistryError,
    resource::ResourceManager,
    serialize::{Decoder, DecoderError, EncoderError},
};

use crate::{
    input::InputManager,
    registry::{component::ComponentRegistryManager, RegistryManager},
    renderer::RendererManager,
    serialize::Encoder,
};

use self::{
    api::{context::Context, ecs::ECS, time::TimeAPI},
    container::ContainerTable,
    entity::EntityTable,
    instance::{SystemInstance, SystemInstanceTable},
    query::QueryTable,
    scheduler::Scheduler,
};

pub mod api;
pub mod archetype;
pub mod container;
pub mod entity;
pub mod error;
pub mod instance;
pub mod query;
pub mod scheduler;
pub mod sparse;
pub mod view;

#[derive(Default)]
pub(crate) struct ECSManager {
    pub(crate) containers: ContainerTable,
    entities: EntityTable,
    queries: QueryTable,
    instances: SystemInstanceTable,
    pub(crate) scheduler: Scheduler,
    global_cycle: u32,
}

pub(crate) struct ECSUpdateContext<'a> {
    pub(crate) registry: &'a mut RegistryManager,
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) system: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) delta_time: f64,
    pub(crate) global_time: f64,
}

impl ECSManager {
    pub(crate) fn save_state(
        &self,
        registry: &ComponentRegistryManager,
        encoder: &mut impl Encoder,
    ) -> Result<(), EncoderError> {
        // encoder.write_u32(self.scenes.len() as u32)?;
        // for scene in self.scenes.values() {
        //     scene.serialize(registry, encoder)?;
        // }
        Ok(())
    }

    pub(crate) fn load_state(
        &mut self,
        registry: &ComponentRegistryManager,
        decoder: &mut impl Decoder,
    ) -> Result<(), DecoderError> {
        // let scenes_count = decoder.read_u32()?;
        // for _ in 0..scenes_count {
        //     let scene = Scene::deserialize(registry, decoder)?;
        //     self.scenes.add(Box::new(scene));
        // }
        Ok(())
    }

    pub(crate) fn on_registry_update(
        &mut self,
        registry: &RegistryManager,
    ) -> Result<(), RegistryError> {
        self.scheduler.on_registry_update(&registry.system);
        self.containers.on_registry_update(&registry.component);
        self.instances
            .on_registry_update(registry, &mut self.entities, &mut self.queries)?;
        Ok(())
    }

    pub(crate) fn update(&mut self, context: ECSUpdateContext) -> Result<(), RegistryError> {
        // Begin frame
        self.scheduler.begin_frame(context.delta_time);

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
                            registry: context.registry,
                            renderer: context.renderer,
                            runtime: context.system,
                            logger: context.logger,
                            time: TimeAPI {
                                delta: context.delta_time,
                                global: context.global_time,
                            },
                        };
                        let ecs = &mut ECS {
                            containers: &mut self.containers,
                            entities: &mut self.entities,
                            queries: &mut self.queries,
                            scheduler: &mut self.scheduler,
                            cycle: self.global_cycle,
                        };
                        // TODO: catch unwind
                        instance.run(ecs, ctx);
                    }
                    SystemInstance::Parallel(instance) => {
                        let ctx = &Context {
                            resource: context.resource,
                            input: context.input,
                            registry: context.registry,
                            renderer: context.renderer,
                            runtime: context.system,
                            logger: context.logger,
                            time: TimeAPI {
                                delta: context.delta_time,
                                global: context.global_time,
                            },
                        };
                        let ecs = &ECS {
                            containers: &mut self.containers,
                            entities: &mut self.entities,
                            queries: &mut self.queries,
                            scheduler: &mut self.scheduler,
                            cycle: self.global_cycle,
                        };
                        // TODO: catch unwind
                        instance.run(ecs, ctx);
                    }
                }

                // Clear filter queries
                for id in &instance.filter_queries {
                    self.queries.filter_queries[id.0].pool.clear();
                }
            } else {
                // TODO: use thread pool
            }
        }

        Ok(())
    }
}
