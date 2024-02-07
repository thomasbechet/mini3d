use core::any::Any;

use crate::{
    component::{NamedComponent, NativeComponent},
    container::{ComponentId, ContainerTable, NativeContainer},
    entity::Entity,
    error::ComponentError,
    query::{EntityQuery, Query},
    registry::Registry,
    scheduler::{Invocation, Scheduler},
};

pub struct ECS<'a> {
    pub(crate) user: &'a mut dyn Any,
    pub(crate) containers: &'a mut ContainerTable,
    pub(crate) registry: &'a mut Registry,
    pub(crate) scheduler: &'a mut Scheduler,
}

macro_rules! for_eachn {
    ($ident:ident, $get:ident, $($ids:ident),*) => {
        #[allow(non_snake_case, unused)]
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn $ident<$($ids: NativeComponent + NamedComponent),*>(
            &mut self,
            callback: impl Fn(Entity, $(&mut $ids),*),
        ) {
            $(let $ids = self.find_component_id($ids::IDENT).unwrap();)*
            let query = Query::default().all(&[$($ids,)*]);
            let ($($ids,)*) = self.containers.$get::<$($ids,)*>($($ids,)*).unwrap();
            let iter = EntityQuery::new(self.registry, &query);
            for e in iter {
                callback(e, $($ids.get_mut(e).unwrap(),)*);
            }
        }
    }
}

impl<'a> ECS<'a> {
    pub fn create(&mut self) -> Entity {
        self.registry.create()
    }

    pub fn destroy(&mut self, e: Entity) {
        self.registry.destroy(e);
    }

    pub fn add<C: NativeComponent + NamedComponent>(&mut self, e: Entity, c: C) {
        if let Some(id) = self.find_component_id(C::IDENT) {
            self.add_from_id(e, id, c).unwrap();
        } else {
            panic!("Component type not found")
        }
    }

    pub fn add_from_id<C: NativeComponent>(
        &mut self,
        e: Entity,
        id: ComponentId,
        c: C,
    ) -> Result<(), ComponentError> {
        self.containers.add(e, id, c, self.user)?;
        self.registry.set(e, id);
        C::on_post_added(self, e)?;
        Ok(())
    }

    pub fn remove<C: NamedComponent>(&mut self, e: Entity) {
        let id = self.find_component_id(C::IDENT).unwrap();
        self.remove_from_id(e, id).unwrap();
    }

    pub fn remove_from_id(&mut self, e: Entity, id: ComponentId) -> Result<(), ComponentError> {
        if let Some(post_removed) = self.containers.remove(e, id, self.user)? {
            self.registry.unset(e, id);
            post_removed(self, e)?;
        }
        Ok(())
    }

    pub fn has<C: NamedComponent>(&self, e: Entity) -> bool {
        let id = self.find_component_id(C::IDENT).unwrap();
        self.has_from_id(e, id)
    }

    pub fn has_from_id(&self, e: Entity, id: ComponentId) -> bool {
        self.registry.has(e, id)
    }

    pub fn get<C: NativeComponent + NamedComponent>(&self, e: Entity) -> Option<&C> {
        self.find_component_id(C::IDENT)
            .and_then(|id| self.get_from_id(e, id))
    }

    pub fn get_from_id<C: NativeComponent>(&self, e: Entity, id: ComponentId) -> Option<&C> {
        self.containers.get::<C>(id).unwrap().get(e)
    }

    pub fn get_mut<C: NativeComponent + NamedComponent>(&mut self, e: Entity) -> Option<&mut C> {
        self.find_component_id(C::IDENT)
            .and_then(|id| self.get_mut_from_id(e, id))
    }

    pub fn get_mut_from_id<C: NativeComponent>(
        &mut self,
        e: Entity,
        id: ComponentId,
    ) -> Option<&mut C> {
        self.containers.get_mut::<C>(id).unwrap().get_mut(e)
    }

    pub fn query(&self) {}

    for_eachn!(for_each2, get_many_mut2, C1, C2);
    for_eachn!(for_each3, get_many_mut3, C1, C2, C3);
    for_eachn!(for_each4, get_many_mut4, C1, C2, C3, C4);
    for_eachn!(for_each5, get_many_mut5, C1, C2, C3, C4, C5);
    for_eachn!(for_each6, get_many_mut6, C1, C2, C3, C4, C5, C6);
    for_eachn!(for_each7, get_many_mut7, C1, C2, C3, C4, C5, C6, C7);
    for_eachn!(for_each8, get_many_mut8, C1, C2, C3, C4, C5, C6, C7, C8);

    pub fn entities(&self) -> impl Iterator<Item = Entity> + '_ {
        self.registry.entities()
    }

    pub fn invoke(&mut self, stage: Entity, invocation: Invocation) {
        self.scheduler.invoke(stage, invocation);
    }

    pub fn get_component_id(&self, e: Entity) -> ComponentId {
        self.containers.component_id(e)
    }

    pub fn find_component_id(&self, ident: &str) -> Option<ComponentId> {
        self.find(ident).map(|e| self.get_component_id(e))
    }

    pub fn find(&self, ident: &str) -> Option<Entity> {
        self.containers.find(ident)
    }

    pub fn tick_stage(&self) -> Entity {
        self.scheduler.tick_stage
    }
}
