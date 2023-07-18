use std::{
    cell::{Ref, RefMut},
    collections::HashSet,
};

use crate::{
    ecs::{
        entity::Entity,
        error::SceneError,
        query::Query,
        scene::Scene,
        singleton::{StaticSingletonMut, StaticSingletonRef},
        view::{SceneComponentViewMut, SceneComponentViewRef},
    },
    registry::{
        component::{Component, ComponentId},
        RegistryManager,
    },
    utils::uid::UID,
};

pub struct ExclusiveSceneContext<'a> {
    uid: UID,
    scene: RefMut<'a, Box<Scene>>,
    registry: Ref<'a, RegistryManager>,
    pub(crate) change_scene: &'a mut Option<UID>,
    pub(crate) removed_scenes: &'a mut HashSet<UID>,
}

impl<'a> ExclusiveSceneContext<'a> {
    pub fn uid(&self) -> UID {
        self.uid
    }

    pub fn add_entity(&mut self) -> Entity {
        self.scene.add_entity()
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<(), SceneError> {
        self.scene.remove_entity(entity)
    }

    pub fn add_static_component<C: Component>(
        &mut self,
        entity: Entity,
        component: ComponentId,
        data: C,
    ) -> Result<(), SceneError> {
        self.scene
            .add_static_component(&self.registry.components, entity, component, data)
    }

    pub fn remove_component(
        &mut self,
        entity: Entity,
        component: ComponentId,
    ) -> Result<(), SceneError> {
        self.scene.remove_component(entity, component)
    }

    pub fn view(&self, component: ComponentId) -> Result<SceneComponentViewRef<'_>, SceneError> {
        self.scene.view(component)
    }

    pub fn view_mut(
        &self,
        component: ComponentId,
    ) -> Result<SceneComponentViewMut<'_>, SceneError> {
        self.scene.view_mut(component)
    }

    pub fn query(&self, components: &[ComponentId]) -> Query<'_> {
        self.scene.query(components)
    }

    pub fn add_singleton<C: Component>(
        &mut self,
        component: ComponentId,
        data: C,
    ) -> Result<(), SceneError> {
        self.scene.add_static_singleton(component, data)
    }

    pub fn remove_singleton(&mut self, component: ComponentId) -> Result<(), SceneError> {
        self.scene.remove_singleton(component)
    }

    pub fn get_singleton<C: Component>(
        &self,
        component: ComponentId,
    ) -> Result<Option<StaticSingletonRef<'_, C>>, SceneError> {
        self.scene.get_static_singleton(component)
    }

    pub fn get_singleton_mut<C: Component>(
        &self,
        component: ComponentId,
    ) -> Result<Option<StaticSingletonMut<'_, C>>, SceneError> {
        self.scene.get_static_singleton_mut(component)
    }
}

pub struct ParallelSceneContext<'a> {
    uid: UID,
    scene: Ref<'a, Box<Scene>>,
    registry: Ref<'a, RegistryManager>,
}

impl<'a> ParallelSceneContext<'a> {
    pub fn view(&self, component: ComponentId) -> Result<SceneComponentViewRef<'_>, SceneError> {
        self.scene.view(component)
    }

    pub fn view_mut(
        &self,
        component: ComponentId,
    ) -> Result<SceneComponentViewMut<'_>, SceneError> {
        self.scene.view_mut(component)
    }

    pub fn query(&self, components: &[ComponentId]) -> Query<'_> {
        self.scene.query(components)
    }
}
