use std::{cell::{Ref, RefMut}, ops::{Deref, DerefMut}};

use anyhow::Result;
use serde::{Serialize, Deserialize};

use super::entity::Entity;


pub struct EntityResolver;
pub struct ComponentContext;

pub trait Component: Serialize + for<'de> Deserialize<'de> + 'static {
    fn on_construct(&mut self, _entity: Entity, _ctx: &mut ComponentContext) -> Result<()> { Ok(()) }
    fn on_destruct(&mut self, _entity: Entity, _ctx: &mut ComponentContext) -> Result<()> { Ok(()) }
    fn resolve_entities(&mut self, _resolver: &EntityResolver) -> Result<()> { Ok(()) }
}

pub struct ComponentRef<'a, C: Component> {
    pub(crate) components: Ref<'a, Vec<C>>,
    pub(crate) index: usize,
}

impl<C: Component> Deref for ComponentRef<'_, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.components[self.index]
    }
}

pub struct ComponentMut<'a, C: Component> {
    pub(crate) components: RefMut<'a, Vec<C>>,
    pub(crate) index: usize,
}

impl<C: Component> Deref for ComponentMut<'_, C> {
    type Target = C;
    fn deref(&self) -> &C {
        &self.components[self.index]
    }
}

impl<C: Component> DerefMut for ComponentMut<'_, C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.components[self.index]
    }
}