use core::any::Any;

use crate::{
    component::{NamedComponent, SingleComponent},
    container::{ComponentId, ContainerTable, SingleContainer},
    entity::{Entity, EntityTable},
    error::ComponentError,
    scheduler::{Invocation, Scheduler},
};

pub struct ECS<'a> {
    pub(crate) user: &'a mut dyn Any,
    pub(crate) entities: &'a mut EntityTable,
    pub(crate) containers: &'a mut ContainerTable,
    pub(crate) scheduler: &'a mut Scheduler,
}

impl<'a> ECS<'a> {
    pub fn create(&mut self) -> Entity {
        self.entities.create()
    }

    pub fn destroy(&mut self, e: Entity) {
        self.entities.destroy(e);
    }

    pub fn add<C: SingleComponent + NamedComponent>(&mut self, e: Entity, c: C) {
        if let Some(id) = self.find_component_id(C::IDENT) {
            self.add_id(e, id, c).unwrap();
        } else {
            panic!("Component type not found")
        }
    }

    pub fn add_id<C: SingleComponent>(
        &mut self,
        e: Entity,
        id: ComponentId,
        c: C,
    ) -> Result<(), ComponentError> {
        self.containers.add(e, id, c, self.user)?;
        C::on_post_added(self, e)?;
        Ok(())
    }

    pub fn remove<C: NamedComponent>(&mut self, e: Entity) {
        let id = self.find_component_id(C::IDENT).unwrap();
        self.remove_id(e, id).unwrap();
    }

    pub fn remove_id(&mut self, e: Entity, id: ComponentId) -> Result<(), ComponentError> {
        if let Some(post_removed) = self.containers.remove(e, id, self.user)? {
            post_removed(self, e)?;
        }
        Ok(())
    }

    pub fn has<C: NamedComponent>(&self, e: Entity) -> bool {
        let id = self.find_component_id(C::IDENT).unwrap();
        self.has_id(e, id)
    }

    pub fn has_id(&self, e: Entity, id: ComponentId) -> bool {
        self.containers.has(e, id)
    }

    pub fn get<C: SingleComponent + NamedComponent>(&self, e: Entity) -> Option<&C> {
        self.find_component_id(C::IDENT)
            .and_then(|id| self.get_id(e, id))
    }

    pub fn get_id<C: SingleComponent>(&self, e: Entity, id: ComponentId) -> Option<&C> {
        self.containers.get::<C>(id).unwrap().get(e)
    }

    pub fn get_mut<C: SingleComponent + NamedComponent>(&mut self, e: Entity) -> Option<&mut C> {
        self.find_component_id(C::IDENT)
            .and_then(|id| self.get_mut_id(e, id))
    }

    pub fn get_mut_id<C: SingleComponent>(&mut self, e: Entity, id: ComponentId) -> Option<&mut C> {
        self.containers.get_mut::<C>(id).unwrap().get_mut(e)
    }

    pub fn query(&self) {}

    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.entities.iter()
    }

    pub fn invoke(&mut self, stage: Entity, invocation: Invocation) {
        self.scheduler.invoke(stage, invocation);
    }

    pub fn component_id(&self, e: Entity) -> ComponentId {
        self.containers.component_id(e)
    }

    pub fn find_component_id(&self, ident: &str) -> Option<ComponentId> {
        self.find(ident).map(|e| self.component_id(e))
    }

    pub fn find(&self, ident: &str) -> Option<Entity> {
        self.containers.find(ident)
    }

    pub fn tick_stage(&self) -> Entity {
        self.scheduler.tick_stage
    }
}
