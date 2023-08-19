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
    pub(crate) registry: &'a RegistryManager,
    pub(crate) archetypes: &'a mut ArchetypeTable,
    pub(crate) components: &'a mut ComponentTable,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) queries: &'a mut QueryTable,
    pub(crate) systems: &'a SystemTable,
    pub(crate) frame_stages: &'a mut VecDeque<SlotId>,
    pub(crate) next_frame_stages: &'a mut VecDeque<SlotId>,
    pub(crate) cycle: u32,
}

impl<'a> ExclusiveSceneContext<'a> {
    pub fn add(&mut self) -> EntityBuilder<'_> {
        EntityBuilder::new(
            &self.registry.components,
            self.archetypes,
            self.entities,
            self.components,
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
    pub(crate) registry: &'a RegistryManager,
    pub(crate) components: &'a ComponentTable,
    pub(crate) entities: &'a EntityTable,
    pub(crate) queries: &'a QueryTable,
    pub(crate) cycle: u32,
}

impl<'a> ParallelSceneContext<'a> {
    pub fn view<H: ComponentHandle>(&self, component: H) -> Result<H::ViewRef<'_>, SceneError> {
        self.components.view(component)
    }

    pub fn view_mut<H: ComponentHandle>(&self, component: H) -> Result<H::ViewMut<'_>, SceneError> {
        self.components.view_mut(component, self.cycle)
    }

    pub fn query(&self, query: QueryId) -> impl Iterator<Item = Entity> + '_ {
        self.queries
            .query_archetypes(query)
            .iter()
            .flat_map(|archetype| self.entities.iter_group_entities(*archetype))
    }

    pub fn filter_query(&self, query: FilterQueryId) -> impl Iterator<Item = Entity> + '_ {
        self.queries.filter_query(query).iter().copied()
    }
}
