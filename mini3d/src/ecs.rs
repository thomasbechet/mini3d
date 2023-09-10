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
    network::server::NetworkServer,
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
        registry::{
            ExclusiveComponentRegistryAPI, ExclusiveRegistryAPI, ExclusiveSystemRegistryAPI,
        },
        renderer::ExclusiveRendererAPI,
        system::ExclusiveSystemAPI,
        time::TimeAPI,
        ExclusiveAPI,
    },
    archetype::ArchetypeTable,
    component::ComponentTable,
    entity::EntityTable,
    error::ECSError,
    instance::{SystemInstanceTable, SystemResult},
    query::QueryTable,
    scheduler::Scheduler,
};

pub mod api;
pub mod archetype;
pub mod component;
pub mod entity;
pub mod error;
pub mod instance;
pub mod query;
pub mod scheduler;
pub mod sparse;
pub mod view;

pub(crate) struct ECSManager {
    pub(crate) components: ComponentTable,
    archetypes: ArchetypeTable,
    entities: EntityTable,
    queries: QueryTable,
    instances: SystemInstanceTable,
    scheduler: Scheduler,
    global_cycle: u32,
}

impl Default for ECSManager {
    fn default() -> Self {
        Self {
            components: ComponentTable::default(),
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

    pub(crate) fn update(&mut self, context: ECSUpdateContext) -> SystemResult {
        // Begin frame
        self.scheduler.begin_frame(context.delta_time);

        // Update cycle
        // Run stages
        // TODO: protect against infinite loops
        while let Some(instances) = self.scheduler.next_node() {
            if instances.len() == 1 {
                // Exclusive
                let instance = instances[0];
                // Build node API
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
                        },
                        components: ExclusiveComponentRegistryAPI {
                            manager: &mut context.registry.components,
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
                    time: TimeAPI {
                        delta: context.delta_time,
                        global: context.global_time,
                    },
                };
                let ecs = &mut ExclusiveECS {
                    archetypes: &mut self.archetypes,
                    components: &mut self.components,
                    entities: &mut self.entities,
                    queries: &mut self.queries,
                    scheduler: &mut self.scheduler,
                    cycle: self.global_cycle,
                };
                self.instances[instance].run_exclusive(ecs, api)?;
            } else {
                // Parallel
                // TODO: use thread pool
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
                    time: TimeAPI {
                        delta: context.delta_time,
                        global: context.global_time,
                    },
                };
                let ecs = &mut ParallelECS {
                    components: &mut self.components,
                    entities: &mut self.entities,
                    queries: &mut self.queries,
                    cycle: self.global_cycle,
                };
                todo!()
            }

            // Check for registry updates
            if context.registry.components.updated {
                self.instances.on_registry_update(&context.registry);
            }
        }

        // Synchronize with registry

        Ok(())
    }
}
