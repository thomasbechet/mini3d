use alloc::boxed::Box;

use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};
use crate::slot_map_key;
use crate::utils::prng::PCG32;
use crate::utils::slotmap::{Key, SlotMap};
use crate::utils::uid::ToUID;

use self::container::{AnyNativeContainer, NativeContainer};
use self::error::ResourceError;
use self::handle::{ResourceHandle, ResourceName, ToResourceHandle};
use self::key::{ResourceSlotKey, ResourceTypeKey};

pub mod container;
pub mod error;
pub mod handle;
pub mod key;
pub mod resource;

pub use resource::*;

slot_map_key!(ResourceEntryKey);

#[derive(Debug)]
pub struct ResourceInfo<'a> {
    pub name: &'a str,
    pub ty: ResourceTypeHandle,
    pub ref_count: u32,
    pub handle: ResourceHandle,
}

pub(crate) struct ResourceEntry {
    name: ResourceName,
    handle: ResourceHandle,
    ty: ResourceTypeHandle,
    ref_count: u32,
}

pub(crate) enum ResourceContainer {
    Native(Box<dyn AnyNativeContainer>),
}

impl ResourceContainer {
    pub(crate) fn iter_slot_keys(&self) -> impl Iterator<Item = ResourceSlotKey> + '_ {
        match self {
            ResourceContainer::Native(container) => container.iter_keys().map(|(_, key)| key),
        }
    }

    pub(crate) fn iter_entry_keys(&self) -> impl Iterator<Item = ResourceEntryKey> + '_ {
        match self {
            ResourceContainer::Native(container) => container.iter_keys().map(|(key, _)| key),
        }
    }

    pub(crate) fn get_entry_key(&self, slot: ResourceSlotKey) -> Option<ResourceEntryKey> {
        match self {
            ResourceContainer::Native(container) => container.get_entry_key(slot),
        }
    }
}

pub struct ResourceManager {
    containers: SlotMap<ResourceTypeKey, ResourceContainer>,
    entries: SlotMap<ResourceEntryKey, ResourceEntry>,
    type_container_key: ResourceTypeKey,
    meta_type: ResourceTypeHandle, // Meta resource type definition
    prng: PCG32,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self {
            containers: Default::default(),
            entries: Default::default(),
            type_container_key: ResourceTypeKey::null(),
            meta_type: ResourceTypeHandle::null(),
            prng: PCG32::new(1234),
        }
    }
}

impl ResourceManager {
    pub(crate) fn define_meta_type(&mut self) {
        // Create container
        self.type_container_key =
            self.containers
                .add(ResourceContainer::Native(Box::new(NativeContainer::<
                    ResourceType,
                >::with_capacity(
                    128
                ))));
        // Create meta type entry
        let entry_key = self.entries.add(ResourceEntry {
            name: ResourceName::new(ResourceType::NAME),
            ty: ResourceTypeHandle::null(),
            handle: ResourceHandle::null(),
            ref_count: 1, // Keep it alive (reference itslef)
        });
        // Create meta type data
        let meta_type_data_slot =
            match &mut self.containers.get_mut(self.type_container_key).unwrap() {
                ResourceContainer::Native(container) => container
                    .as_any_mut()
                    .downcast_mut::<NativeContainer<ResourceType>>()
                    .unwrap()
                    .add(
                        ResourceType {
                            kind: Default::default(),
                            type_key: self.type_container_key,
                        },
                        entry_key,
                    ),
                _ => unreachable!(),
            };

        // Update entry with type and data slot (itself)
        let handle = ResourceHandle::new(self.type_container_key, meta_type_data_slot);
        self.entries[entry_key].handle = handle;
        self.meta_type = handle.into();
        self.entries[entry_key].ty = self.meta_type;
    }

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        Ok(())
    }

    fn get_type(&self, ty: ResourceTypeHandle) -> Option<&ResourceType> {
        let handle = ty.to_handle();
        match self.containers[self.type_container_key] {
            ResourceContainer::Native(ref container) => container
                .as_any()
                .downcast_ref::<NativeContainer<ResourceType>>()
                .unwrap()
                .get(handle.slot_key()),
            _ => unreachable!(),
        }
    }

    fn create_container(&mut self, ty: ResourceTypeHandle) -> ResourceTypeKey {
        let container = self.get_type(ty).unwrap().create_container();
        let type_key = self.containers.add(container);
        // Save container id
        match &mut self.containers[self.type_container_key] {
            ResourceContainer::Native(container) => {
                container
                    .as_any_mut()
                    .downcast_mut::<NativeContainer<ResourceType>>()
                    .unwrap()
                    .get_mut_unchecked(ty.to_handle().slot_key())
                    .type_key = type_key;
            }
            _ => unreachable!(),
        }
        type_key
    }

    fn get_entry_key(&self, handle: ResourceHandle) -> Option<ResourceEntryKey> {
        self.containers
            .get(handle.type_key())
            .and_then(|container| container.get_entry_key(handle.slot_key()))
    }

    pub(crate) fn create<R: Resource>(
        &mut self,
        name: Option<&str>,
        ty: ResourceTypeHandle,
        data: R,
    ) -> Result<ResourceHandle, ResourceError> {
        // Check existing type and container
        if let Some(resource) = self.get_type(ty) {
            if resource.type_key.is_null() {
                self.create_container(ty);
            }
        } else {
            return Err(ResourceError::ResourceTypeNotFound);
        }
        // Check duplicated entry or generate new key
        let name = if let Some(name) = name {
            // Find existing
            if self.find(name).is_some() {
                return Err(ResourceError::DuplicatedAssetEntry);
            }
            ResourceName::new(name)
        } else {
            // Generate random key
            ResourceName::random(&mut self.prng)
        };
        let type_key = self.get_type(ty).unwrap().type_key;
        // Create new entry
        let entry_key: ResourceEntryKey = self.entries.add(ResourceEntry {
            name,
            ty,
            handle: ResourceHandle::null(),
            ref_count: 0,
        });
        // Allocate in container
        // Load asset
        // TODO: preload resource in container ? wait for read ? define proper strategy
        // TODO: check max size
        let slot_key = match &mut self.containers[type_key] {
            ResourceContainer::Native(container) => container
                .as_any_mut()
                .downcast_mut::<NativeContainer<R>>()
                .expect("Invalid native resource container")
                .add(data, entry_key),
            _ => todo!(),
        };

        let handle = ResourceHandle::new(type_key, slot_key);
        // Update resource entry
        self.entries[entry_key].handle = handle;
        Ok(handle)
    }

    pub(crate) fn create_resource_type(
        &mut self,
        name: Option<&str>,
        data: ResourceType,
    ) -> Result<ResourceTypeHandle, ResourceError> {
        self.create(name, self.meta_type, data)
            .map(|handle| handle.into())
    }

    pub(crate) fn native<R: Resource>(
        &self,
        handle: impl ToResourceHandle,
    ) -> Result<&R, ResourceError> {
        let handle = handle.to_handle();
        self.containers
            .get(handle.type_key())
            .and_then(|container| match container {
                ResourceContainer::Native(container) => container
                    .as_any()
                    .downcast_ref::<NativeContainer<R>>()
                    .expect("Invalid native resource container")
                    .get(handle.slot_key()),
                _ => unreachable!(),
            })
            .ok_or(ResourceError::ResourceNotFound)
    }

    pub(crate) fn native_mut<R: Resource>(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> Result<&mut R, ResourceError> {
        let handle = handle.to_handle();
        self.containers
            .get_mut(handle.type_key())
            .and_then(|container| match container {
                ResourceContainer::Native(container) => container
                    .as_any_mut()
                    .downcast_mut::<NativeContainer<R>>()
                    .expect("Invalid native resource container")
                    .get_mut(handle.slot_key()),
                _ => unreachable!(),
            })
            .ok_or(ResourceError::ResourceNotFound)
    }

    pub(crate) fn native_unchecked<R: Resource>(&self, handle: impl ToResourceHandle) -> &R {
        let handle = handle.to_handle();
        match &self.containers[handle.type_key()] {
            ResourceContainer::Native(container) => container
                .as_any()
                .downcast_ref::<NativeContainer<R>>()
                .unwrap()
                .get_unchecked(handle.slot_key()),
            _ => unreachable!(),
        }
    }

    pub(crate) fn native_mut_unchecked<R: Resource>(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> &mut R {
        let handle = handle.to_handle();
        match &mut self.containers[handle.type_key()] {
            ResourceContainer::Native(container) => container
                .as_any_mut()
                .downcast_mut::<NativeContainer<R>>()
                .unwrap()
                .get_mut_unchecked(handle.slot_key()),
            _ => unreachable!(),
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = ResourceHandle> + '_ {
        self.entries.values().map(|entry| entry.handle)
    }

    pub(crate) fn iter_typed(
        &self,
        ty: ResourceTypeHandle,
    ) -> Box<dyn Iterator<Item = ResourceHandle> + '_> {
        let type_key = ty.to_handle().type_key();
        if let Some(container) = self.containers.get(type_key) {
            Box::new(
                container
                    .iter_slot_keys()
                    .map(move |slot_key| ResourceHandle::new(type_key, slot_key)),
            )
        } else {
            Box::new(core::iter::empty())
        }
    }

    pub(crate) fn iter_native_mut<R: Resource>(
        &mut self,
        ty: ResourceTypeHandle,
    ) -> Box<dyn Iterator<Item = (ResourceHandle, &'_ mut R)> + '_> {
        if let Some(type_key) = self.get_type(ty).map(|ty| ty.type_key) {
            if !type_key.is_null() {
                match &mut self.containers[type_key] {
                    ResourceContainer::Native(ref mut container) => {
                        return Box::new(
                            container
                                .as_any_mut()
                                .downcast_mut::<NativeContainer<R>>()
                                .unwrap()
                                .iter_mut()
                                .map(move |(key, value)| {
                                    (ResourceHandle::new(type_key, key), value)
                                }),
                        )
                    }
                }
            }
        }
        Box::new(core::iter::empty())
    }

    pub(crate) fn iter_native_values_mut<R: Resource>(
        &mut self,
        ty: ResourceTypeHandle,
    ) -> impl Iterator<Item = &mut R> + '_ {
        self.iter_native_mut(ty).map(|(_, value)| value)
    }

    pub(crate) fn find(&self, name: impl ToUID) -> Option<ResourceHandle> {
        for entry in self.entries.values() {
            if entry.name.to_uid() == name.to_uid() {
                return Some(entry.handle);
            }
        }
        None
    }

    pub(crate) fn find_typed<H: ToResourceHandle>(
        &self,
        name: impl ToUID,
        ty: ResourceTypeHandle,
    ) -> Option<H> {
        self.get_type(ty)
            .map(|ty| ty.type_key)
            .and_then(|type_key| {
                self.containers.get(type_key).and_then(|container| {
                    container.iter_entry_keys().find_map(|entry_key| {
                        if self.entries[entry_key].name.to_uid() == name.to_uid() {
                            Some(H::from_handle(self.entries[entry_key].handle))
                        } else {
                            None
                        }
                    })
                })
            })
    }

    pub(crate) fn find_type(&self, name: impl ToUID) -> Option<ResourceTypeHandle> {
        self.find_typed::<ResourceTypeHandle>(name, self.meta_type)
    }

    pub(crate) fn info(
        &self,
        handle: impl ToResourceHandle,
    ) -> Result<ResourceInfo, ResourceError> {
        let handle = handle.to_handle();
        let entry_key = self
            .get_entry_key(handle)
            .ok_or(ResourceError::ResourceNotFound)?;
        let entry = &self.entries[entry_key];
        Ok(ResourceInfo {
            name: entry.name.as_str(),
            ty: entry.ty,
            ref_count: entry.ref_count,
            handle: handle.to_handle(),
        })
    }

    pub(crate) fn increment_ref(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> Result<(), ResourceError> {
        let entry_key = self
            .get_entry_key(handle.to_handle())
            .ok_or(ResourceError::ResourceNotFound)?;
        self.entries[entry_key].ref_count += 1;
        Ok(())
    }

    pub(crate) fn decrement_ref(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> Result<(), ResourceError> {
        let entry_key = self
            .get_entry_key(handle.to_handle())
            .ok_or(ResourceError::ResourceNotFound)?;
        let entry = &mut self.entries[entry_key];
        if entry.ref_count > 0 {
            entry.ref_count -= 1;
        }
        // TODO: unload ?
        // if !self.entries[id].slot.is_null() {
        //     self.containers
        //         .get_mut(id)
        //         .unwrap()
        //         .remove(self.entries[id].slot);
        // }
        // // Remove entry
        // self.entries.remove(id);
        Ok(())
    }
}
