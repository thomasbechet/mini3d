use anyhow::{Result, anyhow};

use crate::{registry::component::Component, ecs::{world::World, entity::Entity, view::{ComponentViewRef, ComponentViewMut}, query::Query}, uid::UID};
use core::cell::RefCell;
use std::{collections::HashMap, cell::RefMut};

pub struct WorldContext<'a> {
    pub(crate) worlds: &'a mut HashMap<UID, RefCell<Box<World>>>,
    pub(crate) active_world: UID,
    pub(crate) change_world: &'a mut Option<UID>,
}

impl<'a> WorldContext<'a> {

    pub fn add(&mut self, name: &str, world: World) -> Result<()> {
        let uid: UID = name.into();
        if self.worlds.contains_key(&uid) {
            return Err(anyhow!("World with name {} already exists", name));
        }
        self.worlds.insert(uid, RefCell::new(Box::new(world)));
        Ok(())
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

    pub fn active<'c: 'b, 'b>(&'c mut self) -> WorldInstanceContext<'b> {
        WorldInstanceContext {
            world: self.worlds.get(&self.active_world).unwrap().borrow_mut(),
        }
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
}

impl<'a> WorldInstanceContext<'a> {

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