use std::collections::VecDeque;

use crate::{
    ecs::{
        archetype::ArchetypeTable,
        component::ComponentTable,
        entity::{Entity, EntityBuilder, EntityTable},
        error::SceneError,
        query::{FilterQueryId, QueryId, QueryTable},
        scheduler::Invocation,
        system::SystemTable,
    },
    registry::{component::ComponentHandle, RegistryManager},
    utils::{slotmap::SlotId, uid::UID},
};

pub struct ExclusiveSceneContext<'a> {
    registry: &'a RegistryManager,
    archetypes: &'a mut ArchetypeTable,
    components: &'a mut ComponentTable,
    entities: &'a mut EntityTable,
    queries: &'a mut QueryTable,
    systems: &'a mut SystemTable,
    pub(crate) frame_stages: &'a mut VecDeque<SlotId>,
    pub(crate) next_frame_stages: &'a mut VecDeque<SlotId>,
    cycle: u32,
}

impl<'a> ExclusiveSceneContext<'a> {
    pub fn add(&mut self) -> EntityBuilder<'_> {
        EntityBuilder::new(
            &self.registry.components,
            &mut self.archetypes,
            &mut self.entities,
            &mut self.components,
            self.cycle,
        )
    }

    pub fn remove(&mut self, entity: Entity) {
        self.entities
            .remove(entity, self.archetypes, self.components)
    }

    pub fn view<H: ComponentHandle>(&self, component: H) -> Result<H::ViewRef<'_>, SceneError> {
        self.components.view(component)
    }

    pub fn view_mut<H: ComponentHandle>(&self, component: H) -> Result<H::ViewMut<'_>, SceneError> {
        self.components.view_mut(component, self.cycle)
    }

    pub fn invoke(&mut self, stage: UID, invocation: Invocation) -> Result<(), SceneError> {
        let stage = self
            .systems
            .find_stage(stage)
            .ok_or(SceneError::SystemStageNotFound)?;
        match invocation {
            Invocation::Immediate => {
                self.frame_stages.push_front(stage);
            }
            Invocation::EndFrame => {
                self.frame_stages.push_back(stage);
            }
            Invocation::NextFrame => {
                self.next_frame_stages.push_back(stage);
            }
        }
        Ok(())
    }

    pub(crate) fn query(&self, query: QueryId) -> impl Iterator<Item = Entity> + '_ {
        self.queries
            .query_archetypes(query)
            .iter()
            .flat_map(|archetype| self.entities.iter_group_entities(*archetype))
    }

    pub(crate) fn filter_query(&self, query: FilterQueryId) -> impl Iterator<Item = Entity> + '_ {
        self.queries.filter_query(query).iter().copied()
    }
}

pub struct ParallelSceneContext<'a> {
    registry: &'a RegistryManager,
    components: &'a ComponentTable,
    entities: &'a EntityTable,
    queries: &'a QueryTable,
    cycle: u32,
}

impl<'a> ParallelSceneContext<'a> {
    pub fn view<H: ComponentHandle>(&self, component: H) -> Result<H::ViewRef<'_>, SceneError> {
        self.components.view(component)
    }

    pub fn view_mut<H: ComponentHandle>(&self, component: H) -> Result<H::ViewMut<'_>, SceneError> {
        self.components.view_mut(component, self.cycle)
    }

    pub(crate) fn query(&self, query: QueryId) -> impl Iterator<Item = Entity> + '_ {
        self.queries
            .query_archetypes(query)
            .iter()
            .flat_map(|archetype| self.entities.iter_group_entities(*archetype))
    }

    pub(crate) fn filter_query(&self, query: FilterQueryId) -> impl Iterator<Item = Entity> + '_ {
        self.queries.filter_query(query).iter().copied()
    }
}
