use anyhow::{Result, anyhow};

use crate::{registry::{RegistryManager, component::Component}, ecs::{world::World, entity::Entity, view::{ComponentViewRef, ComponentViewMut}, query::Query}, uid::UID};
use core::cell::RefCell;
use std::{collections::HashMap, cell::{RefMut, Ref}};

pub struct WorldManagerContext<'a> {
    registry: &'a RefCell<RegistryManager>,
    worlds: RefMut<'a, HashMap<UID, RefCell<Box<World>>>>,
    active_world: UID,
    change_world: &'a RefCell<Option<UID>>,
}

impl<'a> WorldManagerContext<'a> {

    pub(crate) fn new(
        registry: &'a RefCell<RegistryManager>, 
        worlds: &'a RefCell<HashMap<UID, RefCell<Box<World>>>>,
        active_world: UID,
        change_world: &'a RefCell<Option<UID>>,
    ) -> Self {
        Self { registry, worlds: worlds.borrow_mut(), active_world, change_world }
    }

    pub fn add(&mut self, name: &str, world: World) -> Result<()> {
        let uid: UID = name.into();
        if self.worlds.contains_key(&uid) {
            return Err(anyhow!("World with name {} already exists", name));
        }
        self.worlds.insert(uid, RefCell::new(Box::new(world)));
        Ok(())
    }
    
    pub fn remove(&mut self, uid: UID) -> Result<()> {
        if let Some(change_world) = *self.change_world.borrow() {
            if change_world == uid {
                return Err(anyhow!("Cannot remove world while it is being changed to"));
            }
        }
        if self.worlds.remove(&uid).is_none() {
            return Err(anyhow!("World with uid {} does not exist", uid));
        }
        Ok(())
    }

    pub fn active(self) -> WorldContext<'a> {
        WorldContext::new(self.registry, self.worlds.get(&self.active_world).unwrap())
    }

    pub fn change(&mut self, uid: UID) -> Result<()> {
        if !self.worlds.contains_key(&uid) {
            return Err(anyhow!("World not found"));
        }
        *self.change_world.borrow_mut() = Some(uid);
        Ok(())
    }
}

pub struct WorldContext<'a> {
    registry: Ref<'a, RegistryManager>,
    worlds: Ref<'a, HashMap<UID, RefCell<Box<World>>>>,
    world: RefMut<'a, Box<World>>,
}

impl<'a> WorldContext<'a> {

    pub(crate) fn new(registry: &'a RefCell<RegistryManager>, world: &'a RefCell<Box<World>>) -> Self {
        Self { registry: registry.borrow(), world: world.borrow_mut() }
    }

    pub fn create(&mut self) -> Entity {
        self.world.create()
    }

    pub fn destroy(&mut self, entity: Entity) -> Result<()> {
        self.world.destroy(entity)
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