use crate::{
    ecs::{
        entity::Entity,
        error::SceneError,
        query::Query,
        scene::Scene,
        singleton::{StaticSingletonMut, StaticSingletonRef},
        view::{
            SceneComponentViewMut, SceneComponentViewRef, StaticSceneComponentViewMut,
            StaticSceneComponentViewRef,
        },
    },
    registry::{
        component::{Component, ComponentId},
        RegistryManager,
    },
    utils::uid::UID,
};
use core::cell::RefCell;
use std::{
    cell::{Ref, RefMut},
    collections::{HashMap, HashSet},
};

// pub struct SceneContext<'a> {
//     pub(crate) registry: &'a RefCell<RegistryManager>,
//     pub(crate) scenes: &'a mut HashMap<UID, RefCell<Box<Scene>>>,
//     pub(crate) active_scene: UID,
//     pub(crate) change_scene: &'a mut Option<UID>,
//     pub(crate) removed_scenes: &'a mut HashSet<UID>,
// }

// impl<'a> SceneContext<'a> {
//     /// Applied immediately
//     pub fn add(&mut self, name: &str) -> Result<UID, SceneError> {
//         let uid: UID = name.into();
//         if self.scenes.contains_key(&uid) {
//             return Err(SceneError::DuplicatedScene {
//                 name: name.to_owned(),
//             });
//         }
//         self.scenes
//             .insert(uid, RefCell::new(Box::new(Scene::new(name))));
//         Ok(uid)
//     }

//     /// Applied at the end of the procedure
//     pub fn remove(&mut self, uid: UID) -> Result<(), SceneError> {
//         if let Some(change_scene) = *self.change_scene {
//             if change_scene == uid {
//                 return Err(SceneError::RemoveAndChangeSameScene { uid });
//             }
//         }
//         if !self.scenes.contains_key(&uid) {
//             return Err(SceneError::SceneNotFound { uid });
//         }
//         self.removed_scenes.insert(uid);
//         Ok(())
//     }

//     pub fn active(&mut self) -> SceneInstanceContext<'_> {
//         SceneInstanceContext {
//             uid: self.active_scene,
//             scene: self.scenes.get(&self.active_scene).unwrap().borrow_mut(),
//             registry: self.registry.borrow(),
//         }
//     }

//     pub fn get(&mut self, uid: UID) -> Result<SceneInstanceContext<'_>, SceneError> {
//         if !self.scenes.contains_key(&uid) {
//             return Err(SceneError::SceneNotFound { uid });
//         }
//         Ok(SceneInstanceContext {
//             uid,
//             scene: self.scenes.get(&uid).unwrap().borrow_mut(),
//             registry: self.registry.borrow(),
//         })
//     }

//     /// Applied at the end of the procedure
//     pub fn change(&mut self, uid: UID) -> Result<(), SceneError> {
//         if self.removed_scenes.contains(&uid) {
//             return Err(SceneError::ChangeToRemovedScene { uid });
//         }
//         if !self.scenes.contains_key(&uid) {
//             return Err(SceneError::SceneNotFound { uid });
//         }
//         *self.change_scene = Some(uid);
//         Ok(())
//     }
// }

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
