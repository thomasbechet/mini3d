use core::any::Any;

use alloc::boxed::Box;
use mini3d_derive::Serialize;
use mini3d_utils::{slot_map_key, slotmap::SlotMap};

use crate::{
    component::{
        component::{Component, ComponentStorage},
        identifier::{Identifier, IdentifierContainer},
        ComponentPostCallback, NamedComponent, NativeComponent,
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

pub trait NativeContainer<C: NativeComponent>: Default + Container {
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
    pub(crate) entity: Entity,
}

#[derive(Default)]
pub(crate) struct ContainerTable {
    pub(crate) entries: SlotMap<ComponentId, ContainerEntry>,
    pub(crate) component_id: ComponentId,
    pub(crate) identifier_id: ComponentId,
}

macro_rules! get_many_mutn {
    ($ident:ident, $($ids:ident),*) => {
        #[allow(non_snake_case, unused)]
        #[allow(clippy::too_many_arguments)]
        pub(crate) fn $ident<$($ids: NativeComponent),*>(
            &mut self,
            $($ids: ComponentId),*,
        ) -> Result<($(&mut $ids::Container),*), ComponentError> {
            let mut containers = self
                .entries
                .get_many_mut([$($ids),*])
                .ok_or(ComponentError::EntryNotFound)?
                .into_iter();
            Ok((
                $(containers
                    .next()
                    .unwrap()
                    .container
                    .as_any_mut()
                    .downcast_mut::<$ids::Container>()
                    .unwrap(),
                )*
            ))
        }
    }
}

impl ContainerTable {
    pub(crate) fn setup(ecs: &mut ECS) {
        // Insert containers
        let component_e = ecs.registry.create();
        ecs.containers.component_id = ecs.containers.entries.add(ContainerEntry {
            container: Box::<LinearContainer<Component>>::default(),
            entity: component_e,
        });
        let identifier_e = ecs.registry.create();
        ecs.containers.identifier_id = ecs.containers.entries.add(ContainerEntry {
            container: Box::<IdentifierContainer>::default(),
            entity: identifier_e,
        });

        // Add manually components
        let component_id = ecs.containers.component_id;
        let identifier_id = ecs.containers.identifier_id;
        let component_container = ecs
            .containers
            .get_mut::<Component>(ecs.containers.component_id)
            .unwrap();
        NativeContainer::add(
            component_container,
            component_e,
            Component {
                storage: ComponentStorage::Single,
                id: (component_id, None),
            },
            &mut (),
        )
        .unwrap();
        NativeContainer::add(
            component_container,
            identifier_e,
            Component {
                storage: ComponentStorage::Single,
                id: (identifier_id, None),
            },
            &mut (),
        )
        .unwrap();
        ecs.registry.add_bitset(component_id);
        ecs.registry.add_bitset(identifier_id);
        ecs.registry.set(component_e, component_id);
        ecs.registry.set(identifier_e, component_id);

        // Add manually identifiers
        let identifier_container = ecs
            .containers
            .get_mut::<Identifier>(ecs.containers.identifier_id)
            .unwrap();
        NativeContainer::add(
            identifier_container,
            component_e,
            Identifier::new(Component::IDENT),
            &mut (),
        )
        .unwrap();
        NativeContainer::add(
            identifier_container,
            identifier_e,
            Identifier::new(Identifier::IDENT),
            &mut (),
        )
        .unwrap();
        ecs.registry.set(component_e, identifier_id);
        ecs.registry.set(identifier_e, identifier_id);
    }

    pub(crate) fn add_container(
        &mut self,
        entity: Entity,
        container: Box<dyn Container>,
    ) -> Result<ComponentId, ComponentError> {
        Ok(self.entries.add(ContainerEntry { container, entity }))
    }

    pub(crate) fn remove_container(
        &mut self,
        entity: Entity,
    ) -> Result<ComponentId, ComponentError> {
        let id = self.entries.iter().find_map(|(id, entry)| {
            if entry.entity == entity {
                Some(id)
            } else {
                None
            }
        });
        if let Some(id) = id {
            self.entries.remove(id);
            Ok(id)
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

    pub fn add<C: NativeComponent>(
        &mut self,
        e: Entity,
        id: ComponentId,
        c: C,
        user: &mut dyn Any,
    ) -> Result<(), ComponentError> {
        let container = self.get_mut::<C>(id)?;
        NativeContainer::add(container, e, c, user)?;
        Ok(())
    }

    pub fn remove(
        &mut self,
        e: Entity,
        id: ComponentId,
        user: &mut dyn Any,
    ) -> Result<Option<ComponentPostCallback>, ComponentError> {
        let container = self.get_any_mut(id)?;
        let post_removed = container.remove(e, user)?;
        Ok(post_removed)
    }

    pub(crate) fn get<C: NativeComponent>(
        &self,
        id: ComponentId,
    ) -> Result<&<C as NativeComponent>::Container, ComponentError> {
        if let Some(entry) = self.entries.get(id) {
            return entry
                .container
                .as_any()
                .downcast_ref::<<C as NativeComponent>::Container>()
                .ok_or(ComponentError::InvalidContainerType);
        }
        Err(ComponentError::EntryNotFound)
    }

    pub(crate) fn get_mut<C: NativeComponent>(
        &mut self,
        id: ComponentId,
    ) -> Result<&mut <C as NativeComponent>::Container, ComponentError> {
        if let Some(entry) = self.entries.get_mut(id) {
            return entry
                .container
                .as_any_mut()
                .downcast_mut::<<C as NativeComponent>::Container>()
                .ok_or(ComponentError::InvalidContainerType);
        }
        Err(ComponentError::EntryNotFound)
    }

    pub(crate) fn get_any_mut(
        &mut self,
        id: ComponentId,
    ) -> Result<&mut dyn Container, ComponentError> {
        if let Some(entry) = self.entries.get_mut(id) {
            return Ok(entry.container.as_mut());
        }
        Err(ComponentError::EntryNotFound)
    }

    get_many_mutn!(get_many_mut2, C0, C1);
    get_many_mutn!(get_many_mut3, C0, C1, C2);
    get_many_mutn!(get_many_mut4, C0, C1, C2, C3);
    get_many_mutn!(get_many_mut5, C0, C1, C2, C3, C4);
    get_many_mutn!(get_many_mut6, C0, C1, C2, C3, C4, C5);
    get_many_mutn!(get_many_mut7, C0, C1, C2, C3, C4, C5, C6);
    get_many_mutn!(get_many_mut8, C0, C1, C2, C3, C4, C5, C6, C7);
}
