use crate::{
    logger::LoggerManager,
    platform::PlatformManager,
    resource::ResourceManager,
    serialize::{Decoder, DecoderError, EncoderError},
    utils::slotmap::{SlotId, SlotMap},
};

use crate::{input::InputManager, renderer::RendererManager, serialize::Encoder};

use self::{
    api::{context::Context, time::TimeAPI},
    instance::ECSInstance,
};

pub mod api;
pub mod archetype;
pub mod builder;
pub mod container;
pub mod entity;
pub mod error;
pub mod instance;
pub mod query;
pub mod scheduler;
pub mod sparse;
pub mod system;
pub mod view;

pub(crate) struct ECSInstanceId(pub(crate) SlotId);

#[derive(Default)]
pub(crate) struct ECSManager {
    instances: SlotMap<ECSInstance>,
}

pub(crate) struct ECSUpdateContext<'a> {
    pub(crate) resource: &'a mut ResourceManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) system: &'a mut PlatformManager,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) delta_time: f64,
    pub(crate) global_time: f64,
}

impl ECSManager {
    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        // encoder.write_u32(self.scenes.len() as u32)?;
        // for scene in self.scenes.values() {
        //     scene.serialize(registry, encoder)?;
        // }
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        // let scenes_count = decoder.read_u32()?;
        // for _ in 0..scenes_count {
        //     let scene = Scene::deserialize(registry, decoder)?;
        //     self.scenes.add(Box::new(scene));
        // }
        Ok(())
    }

    pub(crate) fn add(&mut self) -> ECSInstanceId {
        ECSInstanceId(self.instances.add(ECSInstance::default()))
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
                            registry: context.registry,
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
                }
            } else {
                // TODO: use thread pool
            }
        }

        Ok(())
    }
}
