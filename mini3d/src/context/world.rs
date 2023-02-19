use anyhow::{Result, anyhow};

use crate::{registry::{RegistryManager, component::Component}, ecs::{world::World, entity::Entity, view::{ComponentView, ComponentViewMut}, query::Query}, uid::UID};
use core::cell::RefCell;
use std::collections::HashMap;

pub struct WorldManagerContext<'a> {
    registry: &'a RefCell<RegistryManager>,
    worlds: &'a RefCell<HashMap<UID, RefCell<Box<World>>>>,
    active_world: UID,
    change_world: &'a mut Option<UID>,
}

impl<'a> WorldManagerContext<'a> {

    pub(crate) fn new(
        registry: &RefCell<RegistryManager>, 
        worlds: &RefCell<HashMap<UID, RefCell<Box<World>>>>,
        active_world: UID,
        change_world: &mut Option<UID>,
    ) -> Self {
        Self { registry, worlds, active_world, change_world }
    }

    pub fn add(&mut self, name: &str, world: World) -> Result<()> {
        let uid: UID = name.into();
        if self.worlds.borrow().contains_key(&uid) {
            return Err(anyhow!("World with name {} already exists", name));
        }
        self.worlds.borrow_mut().insert(uid, RefCell::new(Box::new(world)));
        Ok(())
    }
    
    pub fn remove(&mut self, uid: UID) -> Result<()> {
        if let Some(change_world) = self.change_world {
            if *change_world == uid {
                return Err(anyhow!("Cannot remove world while it is being changed to"));
            }
        }
        if self.worlds.borrow_mut().remove(&uid).is_none() {
            return Err(anyhow!("World with uid {} does not exist", uid));
        }
        Ok(())
    }

    pub fn active(&self) -> WorldContext<'_> {
        WorldContext::new(self.registry, self.worlds.borrow().get(&self.active_world).unwrap().borrow_mut().as_mut())
    }

    pub fn change(&mut self, uid: UID) -> Result<()> {
        if !self.worlds.borrow().contains_key(&uid) {
            return Err(anyhow!("World not found"));
        }
        *self.change_world = Some(uid);
        Ok(())
    }
}

pub struct WorldContext<'a> {
    registry: &'a RefCell<RegistryManager>,
    world: &'a mut World,
}

impl<'a> WorldContext<'a> {

    pub(crate) fn new(registry: &RefCell<RegistryManager>, world: &mut World) -> Self {
        Self { registry, world }
    }

    pub fn create(&self) -> Entity {
        self.world.create()
    }

    pub fn destroy(&self, entity: Entity) -> Result<()> {
        self.world.destroy(entity)
    }

    pub fn view<C: Component>(&self, component: UID) -> Result<ComponentView<'_, C>> {
        self.world.view(component)
    }

    pub fn view_mut<C: Component>(&self, component: UID) -> Result<ComponentViewMut<'_, C>> {
        self.world.view_mut(component)
    }

    pub fn query(&self, components: &[UID]) -> Query<'_> {
        self.world.query(components)
    }
}