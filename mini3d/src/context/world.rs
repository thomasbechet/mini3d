use anyhow::{Result, anyhow};

use crate::{registry::{component::Component, RegistryManager}, ecs::{world::World, entity::Entity, view::{ComponentViewRef, ComponentViewMut}, query::Query}, uid::UID};
use core::cell::RefCell;
use std::{collections::HashMap, cell::{RefMut, Ref}};

pub struct WorldContext<'a> {
    pub(crate) registry: &'a RefCell<RegistryManager>,
    pub(crate) worlds: &'a mut HashMap<UID, RefCell<Box<World>>>,
    pub(crate) active_world: UID,
    pub(crate) change_world: &'a mut Option<UID>,
}

impl<'a> WorldContext<'a> {

    pub fn add(&mut self, name: &str) -> Result<UID> {
        let uid: UID = name.into();
        if self.worlds.contains_key(&uid) {
            return Err(anyhow!("World with name {} already exists", name));
        }
        self.worlds.insert(uid, RefCell::new(Box::new(World::new(name))));
        Ok(uid)
    }
    
    pub fn remove(&mut self, uid: UID) -> Result<()> {
        if let Some(change_world) = *self.change_world {
            if change_world == uid {
                return Err(anyhow!("Cannot remove world while it is being changed to"));
            }
        }
        if self.worlds.remove(&uid).is_none() {
            return Err(anyhow!("World with uid {} does not exist", uid));
        }
        Ok(())
    }

    pub fn active(&mut self) -> WorldInstanceContext<'_> {
        WorldInstanceContext { world: self.worlds.get(&self.active_world).unwrap().borrow_mut(), registry: self.registry.borrow() }
    }

    pub fn get(&mut self, uid: UID) -> Result<WorldInstanceContext<'_>> {
        if !self.worlds.contains_key(&uid) {
            return Err(anyhow!("World not found"));
        }
        Ok(WorldInstanceContext { world: self.worlds.get(&uid).unwrap().borrow_mut(), registry: self.registry.borrow() })
    }

    pub fn change(&mut self, uid: UID) -> Result<()> {
        if !self.worlds.contains_key(&uid) {
            return Err(anyhow!("World not found"));
        }
        *self.change_world = Some(uid);
        Ok(())
    }
}

pub struct WorldInstanceContext<'a> {
    world: RefMut<'a, Box<World>>,
    registry: Ref<'a, RegistryManager>
}

impl<'a> WorldInstanceContext<'a> {

    pub fn create(&mut self) -> Entity {
        self.world.create()
    }

    pub fn destroy(&mut self, entity: Entity) -> Result<()> {
        self.world.destroy(entity)
    }

    pub fn add<C: Component>(&mut self, entity: Entity, component: UID, data: C) -> Result<()> {
        self.world.add(&self.registry.components, entity, component, data)
    }

    pub fn remove(&mut self, entity: Entity, component: UID) -> Result<()> {
        self.world.remove(entity, component)
    }

    pub fn view<C: Component>(&self, component: UID) -> Result<ComponentViewRef<'_, C>> {
        self.world.view(component)
    }

    pub fn view_mut<C: Component>(&self, component: UID) -> Result<ComponentViewMut<'_, C>> {
        self.world.view_mut(component)
    }

    pub fn query(&self, components: &[UID]) -> Query<'_> {
        self.world.query(components)
    }
}