use crate::{
    input::backend::InputBackend,
    network::backend::NetworkBackend,
    renderer::backend::RendererBackend,
    serialize::{Decoder, DecoderError, EncoderError},
    storage::backend::StorageBackend,
};
use core::cell::RefCell;

use crate::{
    asset::AssetManager,
    input::InputManager,
    registry::{component::ComponentRegistry, RegistryManager},
    renderer::RendererManager,
    serialize::Encoder,
};

use self::{
    archetype::ArchetypeTable, component::ComponentTable, entity::EntityTable, error::SceneError,
    query::QueryTable, scheduler::Scheduler, system::SystemTable,
};

pub mod api;
pub mod archetype;
pub mod component;
pub mod entity;
pub mod error;
pub mod query;
pub mod scheduler;
pub mod sparse;
pub mod system;
pub mod view;

pub(crate) struct ECSManager {
    pub(crate) components: ComponentTable,
    archetypes: ArchetypeTable,
    entities: EntityTable,
    queries: QueryTable,
    systems: SystemTable,
    scheduler: Scheduler,
}

impl Default for ECSManager {
    fn default() -> Self {
        Self {
            components: ComponentTable::default(),
            archetypes: ArchetypeTable::new(),
            entities: EntityTable::default(),
            queries: QueryTable::default(),
            systems: SystemTable::default(),
            scheduler: Scheduler::default(),
        }
    }
}

pub(crate) struct ECSUpdateContext<'a> {
    pub(crate) registry: &'a RefCell<RegistryManager>,
    pub(crate) asset: &'a mut AssetManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) input_backend: &'a mut dyn InputBackend,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) renderer_backend: &'a mut dyn RendererBackend,
    pub(crate) storage_backend: &'a mut dyn StorageBackend,
    pub(crate) network_backend: &'a mut dyn NetworkBackend,
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

    pub(crate) fn update(&mut self, mut context: ECSUpdateContext) -> Result<(), SceneError> {
        self.scheduler.update(
            &mut self.archetypes,
            &mut self.components,
            &mut self.entities,
            &mut self.queries,
            &mut self.systems,
            &mut context,
        )
    }
}
