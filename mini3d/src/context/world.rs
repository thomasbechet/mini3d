use crate::{ecs::{world::World, entity::Entity, view::{ComponentViewRef, ComponentViewMut}, query::Query, reference::{ComponentRef, ComponentMut}, singleton::{SingletonRef, SingletonMut}, error::WorldError}, uid::UID, registry::{RegistryManager, component::Component}};
use core::cell::RefCell;
use std::{collections::{HashMap, HashSet}, cell::{RefMut, Ref}};

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
            return Err(WorldError::DuplicatedWorld { name: name.to_owned() });
        }
        self.worlds.insert(uid, RefCell::new(Box::new(World::new(name))));
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
        WorldInstanceContext { uid: self.active_world, world: self.worlds.get(&self.active_world).unwrap().borrow_mut(), registry: self.registry.borrow() }
    }

    pub fn get(&mut self, uid: UID) -> Result<WorldInstanceContext<'_>, WorldError> {
        if !self.worlds.contains_key(&uid) {
            return Err(WorldError::WorldNotFound { uid });
        }
        Ok(WorldInstanceContext { uid, world: self.worlds.get(&uid).unwrap().borrow_mut(), registry: self.registry.borrow() })
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
    registry: Ref<'a, RegistryManager>
}

impl<'a> WorldInstanceContext<'a> {

    pub fn uid(&self) -> UID {
        self.uid
    }

    pub fn create(&mut self) -> Entity {
        self.world.create()
    }

    pub fn destroy(&mut self, entity: Entity) -> Result<(), WorldError> {
        self.world.destroy(entity)
    }

    pub fn add<C: Component>(&mut self, entity: Entity, component: UID, data: C) -> Result<(), WorldError> {
        self.world.add(&self.registry.components, entity, component, data)
    }

    pub fn remove(&mut self, entity: Entity, component: UID) -> Result<(), WorldError> {
        self.world.remove(entity, component)
    }

    pub fn get<C: Component>(&self, entity: Entity, component: UID) -> Result<Option<ComponentRef<'_, C>>, WorldError> {
        self.world.get(entity, component)
    }

    pub fn get_mut<C: Component>(&self, entity: Entity, component: UID) -> Result<Option<ComponentMut<'_, C>>, WorldError> {
        self.world.get_mut(entity, component)
    }

    pub fn view<C: Component>(&self, component: UID) -> Result<ComponentViewRef<'_, C>, WorldError> {
        self.world.view(component)
    }

    pub fn view_mut<C: Component>(&self, component: UID) -> Result<ComponentViewMut<'_, C>, WorldError> {
        self.world.view_mut(component)
    }

    pub fn query(&self, components: &[UID]) -> Query<'_> {
        self.world.query(components)
    }

    pub fn add_singleton<C: Component>(&mut self, component: UID, data: C) -> Result<(), WorldError> {
        self.world.add_singleton(component, data)
    }

    pub fn remove_singleton(&mut self, component: UID) -> Result<(), WorldError> {
        self.world.remove_singleton(component)
    }

    pub fn get_singleton<C: Component>(&self, component: UID) -> Result<Option<SingletonRef<'_, C>>, WorldError> {
        self.world.get_singleton(component)
    }

    pub fn get_singleton_mut<C: Component>(&self, component: UID) -> Result<Option<SingletonMut<'_, C>>, WorldError> {
        self.world.get_singleton_mut(component)
    }
}