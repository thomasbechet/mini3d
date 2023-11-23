use crate::activity::ActivityInstanceHandle;
use crate::feature::core::resource::{Resource, ResourceType, ResourceTypeHandle};
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};
use crate::slot_map_key;
use crate::utils::prng::PCG32;
use crate::utils::slotmap::{Key, SlotMap};
use crate::utils::uid::ToUID;

use self::container::{NativeContainer, NativeResourceContainer};
use self::error::ResourceError;
use self::handle::{ResourceHandle, ResourceKey, ToResourceHandle};
use self::key::ResourceTypeKey;

pub mod container;
pub mod error;
pub mod handle;
pub mod key;

slot_map_key!(ResourceEntryKey);

#[derive(Debug)]
pub struct ResourceInfo<'a> {
    pub id: &'a str,
    pub ty_name: &'a str,
    pub ty: ResourceTypeHandle,
    pub owner: ActivityInstanceHandle,
    pub ref_count: u32,
    pub handle: ResourceHandle,
}

pub(crate) struct ResourceEntry {
    key: ResourceKey,
    handle: ResourceHandle,
    owner: ActivityInstanceHandle,
    ref_count: u32,
}

pub(crate) enum ResourceContainer {
    Native(Box<dyn NativeContainer>),
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
    pub(crate) fn define_meta_type(&mut self, root: ActivityInstanceHandle) {
        // Create container
        self.type_container_key = self.containers.add(ResourceContainer::Native(Box::new(
            NativeResourceContainer::<ResourceType>::with_capacity(128),
        )));
        // Create meta type entry
        let entry_key = self.entries.add(ResourceEntry {
            key: ResourceKey::new(ResourceType::NAME),
            handle: ResourceHandle::null(),
            owner: root,
            ref_count: 1, // Keep it alive (reference itslef)
        });
        // Create meta type data
        let meta_type_data_slot = match self.containers.get_mut(self.type_container_key).unwrap() {
            ResourceContainer::Native(container) => container
                .as_any_mut()
                .downcast_mut::<NativeResourceContainer<ResourceType>>()
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
        self.entries[entry_key].handle =
            ResourceHandle::new(self.type_container_key, meta_type_data_slot);
    }

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        Ok(())
    }

    fn get_type(&self, ty: ResourceTypeHandle) -> Option<&ResourceType> {
        let handle = ty.to_handle();
        assert!(handle.type_key() == self.type_container_key);
        match self.containers[self.type_container_key] {
            ResourceContainer::Native(ref container) => container
                .as_any()
                .downcast_ref::<NativeResourceContainer<ResourceType>>()
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
                    .downcast_mut::<NativeResourceContainer<ResourceType>>()
                    .unwrap()
                    .get_mut_unchecked(ty.to_handle().slot_key())
                    .type_key = type_key;
            }
            _ => unreachable!(),
        }
        type_key
    }

    pub(crate) fn create<R: Resource>(
        &mut self,
        key: Option<&str>,
        ty: ResourceTypeHandle,
        owner: ActivityInstanceHandle,
        data: R,
    ) -> Result<ResourceHandle, ResourceError> {
        // Check existing type and container
        if let Some(resource) = self.get_type(ty) {
            if resource.type_key.index().is_none() {
                self.create_container(ty);
            }
        } else {
            return Err(ResourceError::ResourceTypeNotFound);
        }
        // Check duplicated entry or generate new key
        let key = if let Some(key) = key {
            // Find existing
            if self.find(key).is_some() {
                return Err(ResourceError::DuplicatedAssetEntry);
            }
            ResourceKey::new(key)
        } else {
            // Generate random key
            ResourceKey::random(&mut self.prng)
        };
        let type_key = self.get_type(ty).unwrap().type_key;
        // Create new entry
        let entry_key: ResourceEntryKey = self.entries.add(ResourceEntry {
            key,
            handle: ResourceHandle::null(),
            owner,
            ref_count: 0,
        });
        // Allocate in container
        // Load asset
        // TODO: preload resource in container ? wait for read ? define proper strategy
        // TODO: check max size
        let slot_key = match &mut self.containers[type_key] {
            ResourceContainer::Native(container) => container
                .as_any_mut()
                .downcast_mut::<NativeResourceContainer<R>>()
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
        key: Option<&str>,
        owner: ActivityInstanceHandle,
        data: ResourceType,
    ) -> Result<ResourceTypeHandle, ResourceError> {
        self.create(key, self.meta_type, owner, data)
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
                    .downcast_ref::<NativeResourceContainer<R>>()
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
                    .downcast_mut::<NativeResourceContainer<R>>()
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
                .downcast_ref::<NativeResourceContainer<R>>()
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
                .downcast_mut::<NativeResourceContainer<R>>()
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
    ) -> impl Iterator<Item = ResourceHandle> + '_ {
        if let Some(container) = self.containers.get(ty.0.type_key()) {
            match container {
                ResourceContainer::Native(container) => container
                    .iter_keys()
                    .map(|(_, slot_key)| ResourceHandle::new(ty.0.type_key(), slot_key)),
                _ => todo!(),
            }
        } else {
            [].iter()
        }
    }

    pub(crate) fn iter_native_mut<R: Resource>(
        &mut self,
        ty: ResourceTypeHandle,
    ) -> impl Iterator<Item = (ResourceHandle, &'_ mut R)> {
        if let Some(ty) = self.get_type(ty) {
            let container = self.containers[ty.type_key]
                .container
                .as_any_mut()
                .downcast_mut::<NativeResourceContainer<R>>()
                .unwrap()
                .iter()
                .map(|(key, value)| (ResourceHandle::new(ty.type_key, key), value));
        } else {
            [].iter()
        }
    }

    pub(crate) fn iter_native_values_mut<R: Resource>(
        &mut self,
        ty: ResourceTypeHandle,
    ) -> impl Iterator<Item = &mut R> + '_ {
        self.iter_native_mut(ty).map(|(_, value)| value)
    }

    pub(crate) fn find(&self, key: impl ToUID) -> Option<ResourceHandle> {
        for (id, entry) in self.entries.iter() {
            if entry.key.to_uid() == key.to_uid() {
                return Some(entry.handle);
            }
        }
        None
    }

    pub(crate) fn find_typed<H: ToResourceHandle>(
        &self,
        key: impl ToUID,
        ty: ResourceTypeHandle,
    ) -> Option<H> {
        self.iter_typed(ty)
            .find(|handle| {
                let entry = &self.entries[*handle];
                entry.key.to_uid() == key.to_uid()
            })
            .map(|handle| H::from_handle(handle))
    }

    pub(crate) fn find_type(&self, key: impl ToUID) -> Option<ResourceTypeHandle> {
        self.find_typed::<ResourceTypeHandle>(key, self.meta_type)
    }

    pub(crate) fn info(
        &self,
        handle: impl ToResourceHandle,
    ) -> Result<ResourceInfo, ResourceError> {
        let handle = handle.to_handle();
        self.containers
            .get(handle.type_key())
            .map(|container| match container {
                ResourceContainer::Native(container) => container.get_entry_key(handle.slot_key()),
                _ => todo!(),
            })
            .self
            .entries
            .get(handle)
            .map(|entry| ResourceInfo {
                id: entry.key.as_str(),
                ty_name: self.entries[entry.ty.0 .0].key.as_str(),
                ty: entry.ty,
                owner: entry.owner,
                ref_count: entry.ref_count,
                handle: handle.to_handle(),
            })
            .ok_or(ResourceError::ResourceNotFound)
    }

    pub(crate) fn increment_ref(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> Result<(), ResourceError> {
        let id = handle.to_handle();
        let entry = self
            .entries
            .get_mut(id)
            .ok_or(ResourceError::ResourceNotFound)?;
        entry.ref_count += 1;
        Ok(())
    }

    pub(crate) fn decrement_ref(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> Result<(), ResourceError> {
        let id = handle.to_handle();
        let entry = self
            .entries
            .get_mut(id)
            .ok_or(ResourceError::ResourceNotFound)?;
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
