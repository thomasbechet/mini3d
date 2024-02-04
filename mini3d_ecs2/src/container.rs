use core::any::Any;

use alloc::boxed::Box;
use mini3d_derive::Serialize;
use mini3d_utils::{slot_map_key, slotmap::SlotMap};

use crate::{
    bitset::{BitIndex, Bitset},
    component::{
        component::{Component, ComponentStorage},
        identifier::{Identifier, IdentifierContainer},
        ComponentPostCallback, NamedComponent, SingleComponent,
    },
    ecs::ECS,
    entity::Entity,
    error::ComponentError,
};

pub use self::linear::LinearContainer;

pub mod linear;
pub mod sparse;

pub trait Container {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn add(
        &mut self,
        entity: Entity,
        user: &mut dyn Any,
    ) -> Result<Option<ComponentPostCallback>, ComponentError>;
    fn remove(
        &mut self,
        entity: Entity,
        user: &mut dyn Any,
    ) -> Result<Option<ComponentPostCallback>, ComponentError>;
}

pub trait SingleContainer<C: SingleComponent>: Default + Container {
    fn get(&self, entity: Entity) -> Option<&C>;
    fn get_mut(&mut self, entity: Entity) -> Option<&mut C>;
    fn add(
        &mut self,
        entity: Entity,
        component: C,
        user: &mut dyn Any,
    ) -> Result<&mut C, ComponentError>;
    fn remove(&mut self, entity: Entity, user: &mut dyn Any) -> Result<(), ComponentError>;
}

slot_map_key!(ComponentId);

#[derive(Default, Serialize, PartialEq, Eq)]
pub(crate) enum ComponentKind {
    Native,
    Dynamic,
    Raw,
    #[default]
    Tag,
}

pub(crate) struct ContainerEntry {
    pub(crate) container: Box<dyn Container>,
    pub(crate) bitset: Bitset,
    pub(crate) entity: Entity,
}

#[derive(Default)]
pub(crate) struct ContainerTable {
    pub(crate) entries: SlotMap<ComponentId, ContainerEntry>,
    pub(crate) component_id: ComponentId,
    pub(crate) identifier_id: ComponentId,
}

impl ContainerTable {
    pub(crate) fn setup(ecs: &mut ECS) {
        // Insert containers
        let component_e = ecs.entities.create();
        ecs.containers.component_id = ecs.containers.entries.add(ContainerEntry {
            container: Box::<LinearContainer<Component>>::default(),
            bitset: Default::default(),
            entity: component_e,
        });
        let identifier_e = ecs.entities.create();
        ecs.containers.identifier_id = ecs.containers.entries.add(ContainerEntry {
            container: Box::<IdentifierContainer>::default(),
            bitset: Default::default(),
            entity: identifier_e,
        });

        // Add manually components
        let component_id = ecs.containers.component_id;
        let identifier_id = ecs.containers.identifier_id;
        let component_container = ecs
            .containers
            .get_mut::<Component>(ecs.containers.component_id)
            .unwrap();
        SingleContainer::add(
            component_container,
            component_e,
            Component {
                storage: ComponentStorage::Single,
                id: (component_id, None),
            },
            &mut (),
        )
        .unwrap();
        SingleContainer::add(
            component_container,
            identifier_e,
            Component {
                storage: ComponentStorage::Single,
                id: (identifier_id, None),
            },
            &mut (),
        )
        .unwrap();
        ecs.containers.entries[component_id]
            .bitset
            .set(component_e.index() as BitIndex, true);
        ecs.containers.entries[ecs.containers.identifier_id]
            .bitset
            .set(identifier_e.index() as BitIndex, true);

        // Add manually identifiers
        let identifier_container = ecs
            .containers
            .get_mut::<Identifier>(ecs.containers.identifier_id)
            .unwrap();
        SingleContainer::add(
            identifier_container,
            component_e,
            Identifier::new(Component::IDENT),
            &mut (),
        )
        .unwrap();
        SingleContainer::add(
            identifier_container,
            identifier_e,
            Identifier::new(Identifier::IDENT),
            &mut (),
        )
        .unwrap();
        ecs.containers.entries[identifier_id]
            .bitset
            .set(component_e.index() as BitIndex, true);
        ecs.containers.entries[ecs.containers.identifier_id]
            .bitset
            .set(identifier_e.index() as BitIndex, true);
    }

    pub(crate) fn add_container(
        &mut self,
        entity: Entity,
        container: Box<dyn Container>,
    ) -> Result<ComponentId, ComponentError> {
        Ok(self.entries.add(ContainerEntry {
            container,
            bitset: Default::default(),
            entity,
        }))
    }

    pub(crate) fn remove_container(&mut self, entity: Entity) -> Result<(), ComponentError> {
        let id = self.entries.iter().find_map(|(id, entry)| {
            if entry.entity == entity {
                Some(id)
            } else {
                None
            }
        });
        if let Some(id) = id {
            self.entries.remove(id);
            Ok(())
        } else {
            Err(ComponentError::EntryNotFound)
        }
    }

    pub(crate) fn component_id(&self, e: Entity) -> ComponentId {
        self.entries[self.component_id]
            .container
            .as_any()
            .downcast_ref::<LinearContainer<Component>>()
            .unwrap()
            .get(e)
            .unwrap()
            .id
            .0
    }

    pub(crate) fn find(&self, ident: &str) -> Option<Entity> {
        self.entries[self.identifier_id]
            .container
            .as_any()
            .downcast_ref::<IdentifierContainer>()
            .unwrap()
            .find(ident)
    }

    pub fn add<C: SingleComponent>(
        &mut self,
        e: Entity,
        id: ComponentId,
        c: C,
        user: &mut dyn Any,
    ) -> Result<(), ComponentError> {
        let container = self.get_mut::<C>(id)?;
        SingleContainer::add(container, e, c, user)?;
        self.entries[id].bitset.set(e.index() as BitIndex, true);
        Ok(())
    }

    pub fn remove(
        &mut self,
        e: Entity,
        id: ComponentId,
        user: &mut dyn Any,
    ) -> Result<Option<ComponentPostCallback>, ComponentError> {
        let container = self.get_mut_any(id)?;
        let post_removed = container.remove(e, user)?;
        self.entries[id].bitset.set(e.index() as BitIndex, false);
        Ok(post_removed)
    }

    pub(crate) fn get<C: SingleComponent>(
        &self,
        id: ComponentId,
    ) -> Result<&<C as SingleComponent>::Container, ComponentError> {
        if let Some(entry) = self.entries.get(id) {
            return entry
                .container
                .as_any()
                .downcast_ref::<<C as SingleComponent>::Container>()
                .ok_or(ComponentError::InvalidContainerType);
        }
        Err(ComponentError::EntryNotFound)
    }

    pub(crate) fn get_mut<C: SingleComponent>(
        &mut self,
        id: ComponentId,
    ) -> Result<&mut <C as SingleComponent>::Container, ComponentError> {
        if let Some(entry) = self.entries.get_mut(id) {
            return entry
                .container
                .as_any_mut()
                .downcast_mut::<<C as SingleComponent>::Container>()
                .ok_or(ComponentError::InvalidContainerType);
        }
        Err(ComponentError::EntryNotFound)
    }

    pub(crate) fn get_mut_any(
        &mut self,
        id: ComponentId,
    ) -> Result<&mut dyn Container, ComponentError> {
        if let Some(entry) = self.entries.get_mut(id) {
            return Ok(entry.container.as_mut());
        }
        Err(ComponentError::EntryNotFound)
    }

    pub(crate) fn has(&self, e: Entity, id: ComponentId) -> bool {
        self.entries
            .get(id)
            .map(|container| container.bitset.is_set(e.index() as BitIndex))
            .unwrap_or(false)
    }
}
