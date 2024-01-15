use core::{any::Any, cell::UnsafeCell};

use alloc::boxed::Box;
use mini3d_utils::{
    slot_map_key,
    slotmap::{Key, SlotMap},
};

use crate::{
    bitset::Bitset,
    component::{component_type::ComponentType, system::System, system_stage::SystemStage},
    context::Context,
    entity::Entity,
    error::ComponentError,
};

use self::native::NativeSingleContainer;

pub mod native;

pub(crate) trait Container {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn bitset(&self) -> &Bitset;
    fn remove(&mut self, ctx: &mut Context, entity: Entity) -> Result<(), ComponentError>;
}

pub(crate) type ContainerWrapper = Box<UnsafeCell<dyn Container>>;

slot_map_key!(ContainerKey);

pub(crate) struct ContainerEntry {
    pub(crate) container: ContainerWrapper,
    pub(crate) entity: Entity,
}

pub(crate) struct ContainerTable {
    pub(crate) entries: SlotMap<ContainerKey, ContainerEntry>,
    component_type_key: ContainerKey,
    system_key: ContainerKey,
    system_stage_key: ContainerKey,
}

impl ContainerTable {
    fn get_component_type(&mut self, entity: Entity) -> Option<&mut ComponentType> {
        self.entries
            .get_mut(self.component_type_key)
            .unwrap()
            .container
            .get_mut()
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<ComponentType>>()
            .unwrap()
            .get_mut(entity)
    }

    pub(crate) fn system_container(&mut self) -> &mut NativeSingleContainer<System> {
        self.entries
            .get_mut(self.system_key)
            .unwrap()
            .container
            .get_mut()
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<System>>()
            .unwrap()
    }

    pub(crate) fn system_stage_container(&mut self) -> &mut NativeSingleContainer<SystemStage> {
        self.entries
            .get_mut(self.system_stage_key)
            .unwrap()
            .container
            .get_mut()
            .as_any_mut()
            .downcast_mut::<NativeSingleContainer<SystemStage>>()
            .unwrap()
    }

    pub(crate) fn enable_component_type(
        &mut self,
        entity: Entity,
    ) -> Result<ContainerKey, ComponentError> {
        let component = self
            .get_component_type(entity)
            .ok_or(ComponentError::EntryNotFound)?;
        if self.entries.iter().any(|(_, entry)| entry.entity == entity) {
            return Err(ComponentError::DuplicatedEntry);
        }
        let container = component.create_container();
        let key = self.entries.add(ContainerEntry { container, entity });
        self.get_component_type(entity).unwrap().key = key;
        Ok(key)
    }

    pub(crate) fn disable_component_type(
        &mut self,
        ctx: &mut Context,
        key: ContainerKey,
    ) -> Result<(), ComponentError> {
        unimplemented!()
    }

    pub(crate) fn get(&self, key: ContainerKey) -> Option<&dyn Container> {
        self.entries
            .get(key)
            .map(|entry| unsafe { &*entry.container.get() })
    }

    pub(crate) fn get_mut(&mut self, key: ContainerKey) -> Option<&mut dyn Container> {
        self.entries
            .get_mut(key)
            .map(|entry| unsafe { &mut *entry.container.get() })
    }
}

impl Default for ContainerTable {
    fn default() -> Self {
        let mut table = Self {
            entries: SlotMap::with_key(),
            component_type_key: ContainerKey::null(),
            system_key: ContainerKey::null(),
            system_stage_key: ContainerKey::null(),
        };

        // Register component type container
        table.component_type_key = table.entries.add(ContainerEntry {
            container: Box::new(UnsafeCell::new(
                NativeSingleContainer::<ComponentType>::with_capacity(128),
            )),
            entity: Entity::null(),
        });

        table
    }
}
