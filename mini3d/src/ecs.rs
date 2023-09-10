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
    registry::{error::RegistryError, system::SystemRegistry},
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
    container::ContainerTable,
    entity::EntityTable,
    instance::{SystemInstanceTable, SystemResult},
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
    scheduler: Scheduler,
    update_scheduler: bool,
    update_containers: bool,
    update_instances: bool,
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
            update_scheduler: true,
            update_containers: true,
            update_instances: true,
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

    pub(crate) fn on_registry_update(&mut self) {
        self.update_scheduler = true;
        self.update_containers = true;
        self.update_instances = true;
    }

    fn check_scheduler_update(&mut self, registry: &SystemRegistry) -> Result<(), RegistryError> {
        if self.update_scheduler {
            self.scheduler.on_registry_update(registry);
            self.update_scheduler = false;
        }
        Ok(())
    }

    fn check_registry_update(&mut self, registry: &RegistryManager) -> Result<(), RegistryError> {
        if self.update_containers {
            self.containers.on_registry_update(&registry.components);
            self.update_containers = false;
        }
        if self.update_instances {
            self.instances.on_registry_update(
                registry,
                &mut self.containers,
                &mut self.entities,
                &mut self.archetypes,
                &mut self.queries,
            )?;
            self.update_instances = false;
        }
        Ok(())
    }

    pub(crate) fn update(&mut self, context: ECSUpdateContext) -> SystemResult {
        // Check scheduler update
        self.check_scheduler_update(&context.registry.systems)?;
        self.check_registry_update(context.registry)?;

        // Begin frame
        self.scheduler.begin_frame(context.delta_time);

        // Update cycle
        // Run stages
        // TODO: protect against infinite loops
        while let Some(node) = self.scheduler.next_node() {
            // Check registry update
            self.check_registry_update(context.registry)?;

            // Execute node
            if node.count == 1 {
                // Exclusive
                let instance = self.scheduler.instances[node.first];
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
                            updated: &mut self.update_instances,
                        },
                        components: ExclusiveComponentRegistryAPI {
                            manager: &mut context.registry.components,
                            updated: &mut self.update_containers,
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
                    containers: &mut self.containers,
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
                    containers: &mut self.containers,
                    entities: &mut self.entities,
                    queries: &mut self.queries,
                    cycle: self.global_cycle,
                };
                todo!()
            }
        }

        Ok(())
    }
}
