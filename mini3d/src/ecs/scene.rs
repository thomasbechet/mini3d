use crate::serialize::Serialize;
use crate::{
    registry::component::ComponentRegistry,
    serialize::{Decoder, DecoderError, Encoder, EncoderError},
};

use super::archetype::ArchetypeTable;
use super::component::ComponentTable;
use super::entity::EntityTable;
use super::error::SceneError;
use super::query::QueryTable;
use super::scheduler::Scheduler;
use super::system::SystemTable;
use super::ECSUpdateContext;

pub(crate) struct Scene {
    pub(crate) name: String,
    pub(crate) components: ComponentTable,
    archetypes: ArchetypeTable,
    entities: EntityTable,
    queries: QueryTable,
    systems: SystemTable,
    scheduler: Scheduler,
}

impl Scene {
    pub(crate) fn serialize(
        &self,
        registry: &ComponentRegistry,
        encoder: &mut impl Encoder,
    ) -> Result<(), EncoderError> {
        self.name.serialize(encoder)?;
        self.components.serialize(registry, encoder)?;

        // TODO...
        Ok(())
    }

    pub(crate) fn deserialize(
        registry: &ComponentRegistry,
        decoder: &mut impl Decoder,
    ) -> Result<Self, DecoderError> {
        let name = String::deserialize(decoder, &Default::default())?;
        Ok(Self::new(&name))
    }

    pub(crate) fn new(name: &str) -> Scene {
        Scene {
            name: name.to_string(),
            components: ComponentTable::default(),
            archetypes: ArchetypeTable::new(),
            entities: EntityTable::default(),
            queries: QueryTable::default(),
            systems: SystemTable::default(),
            scheduler: Scheduler::default(),
        }
    }

    pub(crate) fn update(&mut self, context: &mut ECSUpdateContext) -> Result<(), SceneError> {
        self.scheduler.update(
            &mut self.archetypes,
            &mut self.components,
            &mut self.entities,
            &mut self.queries,
            &mut self.systems,
            context,
        )
    }

    // TODO: pub(crate) fn transfer -> transfer scene to scene
    // TODO: pub(crate) fn export -> scene to prefab
    // TODO: pub(crate) fn import -> prefab to scene
}
