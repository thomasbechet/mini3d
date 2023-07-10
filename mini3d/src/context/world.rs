use crate::{
    ecs::{
        entity::Entity,
        error::WorldError,
        query::Query,
        singleton::{StaticSingletonMut, StaticSingletonRef},
        view::{
            AnyComponentViewMut, AnyComponentViewRef, StaticComponentViewMut,
            StaticComponentViewRef,
        },
        world::World,
    },
    registry::{component::Component, RegistryManager},
    uid::UID,
};
use core::cell::RefCell;
use std::{
    cell::{Ref, RefMut},
    collections::{HashMap, HashSet},
};

pub struct WorldContext<'a> {
    pub(crate) registry: &'a RefCell<RegistryManager>,
    pub(crate) worlds: &'a mut HashMap<UID, RefCell<Box<World>>>,
    pub(crate) active_world: UID,
    pub(crate) change_world: &'a mut Option<UID>,
    pub(crate) removed_worlds: &'a mut HashSet<UID>,
}

impl<'a> WorldContext<'a> {
    /// Applied immediately
    pub fn add(&mut self, name: &str) -> Result<UID, WorldError> {
        let uid: UID = name.into();
        if self.worlds.contains_key(&uid) {
            return Err(WorldError::DuplicatedWorld {
                name: name.to_owned(),
            });
        }
        self.worlds
            .insert(uid, RefCell::new(Box::new(World::new(name))));
        Ok(uid)
    }

    /// Applied at the end of the procedure
    pub fn remove(&mut self, uid: UID) -> Result<(), WorldError> {
        if let Some(change_world) = *self.change_world {
            if change_world == uid {
                return Err(WorldError::RemoveAndChangeSameWorld { uid });
            }
        }
        if !self.worlds.contains_key(&uid) {
            return Err(WorldError::WorldNotFound { uid });
        }
        self.removed_worlds.insert(uid);
        Ok(())
    }

    pub fn active(&mut self) -> WorldInstanceContext<'_> {
        WorldInstanceContext {
            uid: self.active_world,
            world: self.worlds.get(&self.active_world).unwrap().borrow_mut(),
            registry: self.registry.borrow(),
        }
    }

    pub fn get(&mut self, uid: UID) -> Result<WorldInstanceContext<'_>, WorldError> {
        if !self.worlds.contains_key(&uid) {
            return Err(WorldError::WorldNotFound { uid });
        }
        Ok(WorldInstanceContext {
            uid,
            world: self.worlds.get(&uid).unwrap().borrow_mut(),
            registry: self.registry.borrow(),
        })
    }

    /// Applied at the end of the procedure
    pub fn change(&mut self, uid: UID) -> Result<(), WorldError> {
        if self.removed_worlds.contains(&uid) {
            return Err(WorldError::ChangeToRemovedWorld { uid });
        }
        if !self.worlds.contains_key(&uid) {
            return Err(WorldError::WorldNotFound { uid });
        }
        *self.change_world = Some(uid);
        Ok(())
    }
}

pub struct WorldInstanceContext<'a> {
    uid: UID,
    world: RefMut<'a, Box<World>>,
    registry: Ref<'a, RegistryManager>,
}

impl<'a> WorldInstanceContext<'a> {
    pub fn uid(&self) -> UID {
        self.uid
    }

    pub fn add_entity(&mut self) -> Entity {
        self.world.add_entity()
    }

    pub fn remove_entity(&mut self, entity: Entity) -> Result<(), WorldError> {
        self.world.remove_entity(entity)
    }

    pub fn add_static_component<C: Component>(
        &mut self,
        entity: Entity,
        component: UID,
        data: C,
    ) -> Result<(), WorldError> {
        self.world
            .add_static_component(&self.registry.components, entity, component, data)
    }

    pub fn remove_component(&mut self, entity: Entity, component: UID) -> Result<(), WorldError> {
        self.world.remove_component(entity, component)
    }

    pub fn static_view<C: Component>(
        &self,
        component: UID,
    ) -> Result<StaticComponentViewRef<'_, C>, WorldError> {
        self.world.static_view(component)
    }

    pub fn any_view(&self, component: UID) -> Result<AnyComponentViewRef<'_>, WorldError> {
        self.world.any_view(component)
    }

    pub fn static_view_mut<C: Component>(
        &self,
        component: UID,
    ) -> Result<StaticComponentViewMut<'_, C>, WorldError> {
        self.world.static_view_mut(component)
    }

    pub fn view_mut_any(&self, component: UID) -> Result<AnyComponentViewMut<'_>, WorldError> {
        self.world.any_view_mut(component)
    }

    pub fn query(&self, components: &[UID]) -> Query<'_> {
        self.world.query(components)
    }

    pub fn add_singleton<C: Component>(
        &mut self,
        component: UID,
        data: C,
    ) -> Result<(), WorldError> {
        self.world.add_static_singleton(component, data)
    }

    pub fn remove_singleton(&mut self, component: UID) -> Result<(), WorldError> {
        self.world.remove_singleton(component)
    }

    pub fn get_singleton<C: Component>(
        &self,
        component: UID,
    ) -> Result<Option<StaticSingletonRef<'_, C>>, WorldError> {
        self.world.get_static_singleton(component)
    }

    pub fn get_singleton_mut<C: Component>(
        &self,
        component: UID,
    ) -> Result<Option<StaticSingletonMut<'_, C>>, WorldError> {
        self.world.get_static_singleton_mut(component)
    }
}
