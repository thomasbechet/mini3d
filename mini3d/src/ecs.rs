use crate::{
    ecs::api::{
        asset::ParallelAssetAPI,
        ecs::ParallelECS,
        input::ParallelInputAPI,
        registry::{ParallelComponentRegistryAPI, ParallelRegistryAPI, ParallelSystemRegistryAPI},
        renderer::ParallelRendererAPI,
        system::ParallelSystemAPI,
        ParallelAPI,
    },
    input::server::InputServer,
    logger::{server::LoggerServer, LoggerManager},
    network::server::NetworkServer,
    registry::error::RegistryError,
    renderer::server::RendererServer,
    serialize::{Decoder, DecoderError, EncoderError},
    storage::server::StorageServer,
    system::{server::SystemServer, SystemManager},
};

use crate::{
    asset::AssetManager,
    input::InputManager,
    registry::{component::ComponentRegistry, RegistryManager},
    renderer::RendererManager,
    serialize::Encoder,
};

use self::{
    api::{
        asset::ExclusiveAssetAPI,
        ecs::ExclusiveECS,
        input::ExclusiveInputAPI,
        logger::{ExclusiveLoggerAPI, ParallelLoggerAPI},
        registry::{
            ExclusiveComponentRegistryAPI, ExclusiveRegistryAPI, ExclusiveSystemRegistryAPI,
        },
        renderer::ExclusiveRendererAPI,
        system::ExclusiveSystemAPI,
        time::TimeAPI,
        ExclusiveAPI,
    },
    archetype::ArchetypeTable,
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

pub(crate) struct ECSManager {
    pub(crate) containers: ContainerTable,
    archetypes: ArchetypeTable,
    entities: EntityTable,
    queries: QueryTable,
    instances: SystemInstanceTable,
    pub(crate) scheduler: Scheduler,
    global_cycle: u32,
}

impl Default for ECSManager {
    fn default() -> Self {
        Self {
            containers: ContainerTable::default(),
            archetypes: ArchetypeTable::new(),
            entities: EntityTable::default(),
            queries: QueryTable::default(),
            instances: SystemInstanceTable::default(),
            scheduler: Scheduler::default(),
            global_cycle: 0,
        }
    }
}

pub(crate) struct ECSUpdateContext<'a> {
    pub(crate) registry: &'a mut RegistryManager,
    pub(crate) asset: &'a mut AssetManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) input_server: &'a mut dyn InputServer,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) renderer_server: &'a mut dyn RendererServer,
    pub(crate) storage_server: &'a mut dyn StorageServer,
    pub(crate) network_server: &'a mut dyn NetworkServer,
    pub(crate) system: &'a mut SystemManager,
    pub(crate) system_server: &'a mut dyn SystemServer,
    pub(crate) logger: &'a mut LoggerManager,
    pub(crate) logger_server: &'a mut dyn LoggerServer,
    pub(crate) delta_time: f64,
    pub(crate) global_time: f64,
}

impl ECSManager {
    pub(crate) fn save_state(
        &self,
        registry: &ComponentRegistry,
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
        registry: &ComponentRegistry,
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
        self.scheduler.on_registry_update(&registry.systems);
        self.containers.on_registry_update(&registry.components);
        self.instances.on_registry_update(
            registry,
            &mut self.containers,
            &mut self.entities,
            &mut self.archetypes,
            &mut self.queries,
        )?;
        Ok(())
    }

    pub(crate) fn update(&mut self, context: ECSUpdateContext) -> Result<(), RegistryError> {
        let mut system_registry_update = false;
        let mut component_registry_update = false;

        // Begin frame
        self.scheduler.begin_frame(context.delta_time);

        // Update cycle
        // Run stages
        // TODO: protect against infinite loops
        loop {
            // Check registry update
            if system_registry_update || component_registry_update {
                system_registry_update = false;
                component_registry_update = false;
                self.on_registry_update(context.registry)?;
            }

            // Acquire next node
            let node = self.scheduler.next_node();
            if node.is_none() {
                break;
            }
            let node = node.unwrap();

            // Execute node
            if node.count == 1 {
                let instance = self.scheduler.instances[node.first];
                match &self
                    .instances
                    .get(instance)
                    .expect("System instance not found")
                    .instance
                {
                    SystemInstance::Exclusive(instance) => {
                        let api = &mut ExclusiveAPI {
                            asset: ExclusiveAssetAPI {
                                manager: context.asset,
                            },
                            input: ExclusiveInputAPI {
                                manager: context.input,
                                server: context.input_server,
                            },
                            registry: ExclusiveRegistryAPI {
                                systems: ExclusiveSystemRegistryAPI {
                                    manager: &mut context.registry.systems,
                                    updated: &mut system_registry_update,
                                },
                                components: ExclusiveComponentRegistryAPI {
                                    manager: &mut context.registry.components,
                                    updated: &mut component_registry_update,
                                },
                            },
                            renderer: ExclusiveRendererAPI {
                                manager: context.renderer,
                                server: context.renderer_server,
                            },
                            system: ExclusiveSystemAPI {
                                server: context.system_server,
                                manager: context.system,
                            },
                            logger: ExclusiveLoggerAPI {
                                server: context.logger_server,
                                manager: context.logger,
                            },
                            time: TimeAPI {
                                delta: context.delta_time,
                                global: context.global_time,
                            },
                        };
                        let ecs = &mut ExclusiveECS {
                            archetypes: &mut self.archetypes,
                            containers: &mut self.containers,
                            entities: &mut self.entities,
                            queries: &mut self.queries,
                            scheduler: &mut self.scheduler,
                            cycle: self.global_cycle,
                        };
                        // TODO: catch unwind
                        instance.run(ecs, api);
                    }
                    SystemInstance::Parallel(instance) => {
                        let api = &mut ParallelAPI {
                            asset: ParallelAssetAPI {
                                manager: context.asset,
                            },
                            input: ParallelInputAPI {
                                manager: context.input,
                            },
                            registry: ParallelRegistryAPI {
                                systems: ParallelSystemRegistryAPI {
                                    manager: &context.registry.systems,
                                },
                                components: ParallelComponentRegistryAPI {
                                    manager: &context.registry.components,
                                },
                            },
                            renderer: ParallelRendererAPI {
                                manager: context.renderer,
                            },
                            system: ParallelSystemAPI {
                                server: context.system_server,
                                manager: context.system,
                            },
                            logger: ParallelLoggerAPI {
                                manager: context.logger,
                            },
                            time: TimeAPI {
                                delta: context.delta_time,
                                global: context.global_time,
                            },
                        };
                        let ecs = &mut ParallelECS {
                            containers: &mut self.containers,
                            entities: &mut self.entities,
                            queries: &mut self.queries,
                            cycle: self.global_cycle,
                        };
                        // TODO: catch unwind
                        instance.run(ecs, api);
                    }
                }
            } else {
                // TODO: use thread pool
            }
        }

        Ok(())
    }
}
