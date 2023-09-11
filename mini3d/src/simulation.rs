use crate::asset::AssetManager;
use crate::ecs::instance::SystemError;
use crate::ecs::scheduler::Invocation;
use crate::ecs::{ECSManager, ECSUpdateContext};
use crate::feature::{component, system};
use crate::input::server::InputServer;
use crate::input::InputManager;
use crate::network::server::NetworkServer;
use crate::physics::PhysicsManager;
use crate::recorder::SimulationRecorder;
use crate::registry::error::RegistryError;
use crate::registry::system::{ExclusiveSystem, SystemOrder, SystemStage};
use crate::registry::RegistryManager;
use crate::renderer::server::RendererServer;
use crate::renderer::RendererManager;
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};
use crate::storage::server::StorageServer;
use crate::storage::StorageManager;
use crate::system::event::SystemEvent;
use crate::system::server::SystemServer;
use crate::system::SystemManager;
use crate::utils::uid::UID;

pub enum ProgressError {
    System(Box<dyn SystemError>),
}

impl core::fmt::Debug for ProgressError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ProgressError::System(err) => write!(f, "System error: {}", err),
        }
    }
}

const MAXIMUM_TIMESTEP: f64 = 1.0 / 20.0;

pub struct ProgressContext<'a> {
    pub input: &'a mut dyn InputServer,
    pub renderer: &'a mut dyn RendererServer,
    pub storage: &'a mut dyn StorageServer,
    pub network: &'a mut dyn NetworkServer,
    pub system: &'a mut dyn SystemServer,
}

pub struct Simulation {
    pub(crate) registry: RegistryManager,
    pub(crate) storage: StorageManager,
    pub(crate) asset: AssetManager,
    pub(crate) input: InputManager,
    pub(crate) ecs: ECSManager,
    pub(crate) renderer: RendererManager,
    pub(crate) physics: PhysicsManager,
    pub(crate) system: SystemManager,
    global_time: f64,
}

impl Simulation {
    fn register_core_features(&mut self) -> Result<(), RegistryError> {
        macro_rules! define_component {
            ($component: ty) => {
                self.registry
                    .components
                    .add_static::<$component>(<$component>::NAME)?;
            };
        }

        macro_rules! define_system_exclusive {
            ($name: literal, $system: ty, $stage: expr) => {
                self.registry.systems.add_static_exclusive::<$system>(
                    $name,
                    $stage,
                    SystemOrder::default(),
                )?;
            };
        }

        macro_rules! define_system_parallel {
            ($name: literal, $system: ty, $stage: expr) => {
                self.registry.systems.add_static_parallel::<$system>(
                    $name,
                    $stage,
                    SystemOrder::default(),
                )?;
            };
        }

        // Define components
        define_component!(component::common::free_fly::FreeFly);
        define_component!(component::common::lifecycle::Lifecycle);
        define_component!(component::common::prefab::Prefab);
        define_component!(component::common::rotator::Rotator);
        define_component!(component::common::script::Script);
        define_component!(component::input::input_table::InputTable);
        define_component!(component::physics::rigid_body::RigidBody);
        define_component!(component::renderer::camera::Camera);
        define_component!(component::renderer::font::Font);
        define_component!(component::renderer::material::Material);
        define_component!(component::renderer::mesh::Mesh);
        define_component!(component::renderer::model::Model);
        define_component!(component::renderer::static_mesh::StaticMesh);
        define_component!(component::renderer::texture::Texture);
        define_component!(component::renderer::tilemap::Tilemap);
        define_component!(component::renderer::tileset::Tileset);
        define_component!(component::renderer::viewport::Viewport);
        define_component!(component::ui::canvas::Canvas);
        define_component!(component::ui::ui_stylesheet::UIStyleSheet);
        define_component!(component::ui::ui_template::UITemplate);
        define_component!(component::ui::ui::UI);
        define_component!(component::ui::ui::UIRenderTarget);
        define_component!(component::scene::hierarchy::Hierarchy);
        define_component!(component::scene::local_to_world::LocalToWorld);
        define_component!(component::scene::transform::Transform);

        // Define systems
        define_system_exclusive!(
            "despawn_entities",
            system::despawn::DespawnEntities,
            SystemStage::UPDATE
        );
        define_system_exclusive!(
            "renderer",
            system::renderer::DespawnRendererEntities,
            SystemStage::UPDATE
        );
        define_system_parallel!(
            "free_fly",
            system::free_fly::FreeFlySystem,
            SystemStage::UPDATE
        );
        define_system_parallel!(
            "rotator",
            system::rotator::RotatorSystem,
            SystemStage::UPDATE
        );
        define_system_parallel!(
            "transform_propagate",
            system::transform::PropagateTransforms,
            SystemStage::UPDATE
        );
        define_system_parallel!("ui_update", system::ui::UpdateUI, SystemStage::UPDATE);
        define_system_exclusive!("ui_render", system::ui::RenderUI, SystemStage::UPDATE);

        Ok(())
    }

    fn setup(&mut self, core_features: bool) {
        // Register core features
        if core_features {
            self.register_core_features()
                .expect("Failed to define core features");
        }
        // Update ECS
        self.ecs
            .scheduler
            .on_registry_update(&self.registry.systems);
        // Setup managers
        self.renderer
            .reload_component_handles(&self.registry.components)
            .expect("Failed to reload component handles");
    }

    pub fn new(core_features: bool) -> Self {
        let mut sim = Self {
            registry: Default::default(),
            storage: Default::default(),
            asset: Default::default(),
            input: Default::default(),
            ecs: Default::default(),
            renderer: Default::default(),
            physics: Default::default(),
            system: Default::default(),
            global_time: 0.0,
        };
        sim.setup(core_features);
        sim
    }

    pub fn register_system<S: ExclusiveSystem>(
        &mut self,
        name: &str,
        stage: &str,
    ) -> Result<(), RegistryError> {
        self.registry
            .systems
            .add_static_exclusive::<S>(name, stage, SystemOrder::default())?;
        self.ecs.on_registry_update(&self.registry)?;
        Ok(())
    }

    pub fn invoke(&mut self, stage: UID, invocation: Invocation) -> Result<(), RegistryError> {
        self.ecs.scheduler.invoke(stage, invocation)
    }

    pub fn save(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.asset.save_state(&self.registry.components, encoder)?;
        self.renderer.save_state(encoder)?;
        self.ecs.save_state(&self.registry.components, encoder)?;
        self.input.save_state(encoder)?;
        self.global_time.serialize(encoder)?;
        Ok(())
    }

    pub fn load(
        &mut self,
        decoder: &mut impl Decoder,
        servers: ProgressContext,
    ) -> Result<(), DecoderError> {
        self.asset.load_state(&self.registry.components, decoder)?;
        self.renderer.load_state(decoder, servers.renderer)?;
        self.ecs.load_state(&self.registry.components, decoder)?;
        self.input.load_state(decoder, servers.input)?;
        self.global_time = Serialize::deserialize(decoder, &Default::default())?;
        Ok(())
    }

    pub fn progress(
        &mut self,
        servers: ProgressContext,
        mut delta_time: f64,
    ) -> Result<(), ProgressError> {
        // ================= PREPARE STAGE ================== //

        // Reset graphics state
        self.renderer.prepare();

        // Compute delta time
        if delta_time > MAXIMUM_TIMESTEP {
            delta_time = MAXIMUM_TIMESTEP; // Slowing down
        }
        // Integrate time
        self.global_time += delta_time;

        // ================= DISPATCH EVENTS STAGE ================= //

        // Prepare input manager
        self.input.prepare_dispatch();
        // Dispatch input events
        self.input.dispatch_events(servers.input);

        // Dispatch system events
        while let Some(event) = servers.system.pool_events() {
            match event {
                SystemEvent::RequestStop => {
                    servers.system.request_stop();
                }
            }
        }

        // Dispatch renderer events
        self.renderer.dispatch_events(servers.renderer);

        // Dispatch network events

        // Dispatch storage events

        // TODO: dispatch more events ...

        // ============ UPDATE/FIXED-UPDATE STAGE =========== //

        self.ecs
            .update(ECSUpdateContext {
                registry: &mut self.registry,
                asset: &mut self.asset,
                input: &mut self.input,
                input_server: servers.input,
                renderer: &mut self.renderer,
                renderer_server: servers.renderer,
                storage_server: servers.storage,
                network_server: servers.network,
                system: &mut self.system,
                system_server: servers.system,
                delta_time,
                global_time: self.global_time,
            })
            .map_err(|err| ProgressError::System(err))?;

        // ================= POST-UPDATE STAGE ================== //
        self.renderer
            .submit_graphics(&mut self.asset, &self.ecs.containers, servers.renderer);

        Ok(())
    }

    pub fn progress_and_record(
        &mut self,
        servers: &mut ProgressContext,
        recorder: &mut SimulationRecorder,
        mut delta_time: f64,
    ) -> Result<(), ProgressError> {
        Ok(())
    }
}
