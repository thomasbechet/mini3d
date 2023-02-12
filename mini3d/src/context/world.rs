use anyhow::Result;

use crate::{registry::{RegistryManager, component::Component}, scene::{world::World, entity::Entity, query::Query, view::{ComponentViewMut, ComponentView}}, uid::UID};
use core::cell::RefCell;

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