use crate::activity::ActivityId;
use crate::feature::core::resource::{Resource, ResourceType, ResourceTypeHandle};
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};
use crate::utils::prng::PCG32;
use crate::utils::slotmap::{SlotId, SlotMap};
use crate::utils::uid::ToUID;

use self::container::{NativeResourceContainer, ResourceContainer};
use self::error::ResourceError;
use self::handle::{ResourceHandle, ToResourceHandle};
use self::iterator::{TypedNativeResourceIteratorMut, TypedResourceIterator};
use self::key::ResourceKey;

pub mod container;
pub mod error;
pub mod handle;
pub mod iterator;
pub mod key;

pub struct ResourceInfo<'a> {
    pub key: &'a str,
    pub ty_name: &'a str,
    pub ty: ResourceTypeHandle,
    pub owner: ActivityId,
    pub ref_count: usize,
    pub handle: ResourceHandle,
}

struct ResourceEntry {
    key: ResourceKey,
    ty: ResourceTypeHandle,
    owner: ActivityId,
    ref_count: usize,
    slot: SlotId,         // Null if not loaded
    prev: ResourceHandle, // Previous entry of same type
    next: ResourceHandle, // Next entry of same type
}

struct ContainerEntry {
    container: Box<dyn ResourceContainer>,
    first: ResourceHandle, // First entry
}

pub struct ResourceManager {
    containers: SlotMap<ContainerEntry>,
    entries: SlotMap<ResourceEntry>,
    type_container: SlotId,        // Containers of resource types
    meta_type: ResourceTypeHandle, // Meta resource type definition
    prng: PCG32,
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self {
            containers: Default::default(),
            entries: Default::default(),
            type_container: SlotId::null(),
            meta_type: ResourceTypeHandle::null(),
            prng: PCG32::new(1234),
        }
    }
}

impl ResourceManager {
    pub(crate) fn define_meta_type(&mut self, root: ActivityId) {
        // Create container
        self.type_container = self.containers.add(ContainerEntry {
            container: Box::new(NativeResourceContainer::<ResourceType>::with_capacity(128)),
            first: ResourceHandle::null(),
        });
        // Create meta type entry
        self.meta_type = ResourceHandle(self.entries.add(ResourceEntry {
            key: ResourceKey::new("resource.type"),
            ty: ResourceTypeHandle::null(),
            owner: root,
            ref_count: 1, // Keep it alive (reference itslef)
            slot: SlotId::null(),
            prev: ResourceHandle::null(),
            next: ResourceHandle::null(),
        }))
        .into();
        // Create meta type data
        let meta_type_data_slot = self
            .containers
            .get_mut(self.type_container)
            .unwrap()
            .container
            .as_any_mut()
            .downcast_mut::<NativeResourceContainer<ResourceType>>()
            .unwrap()
            .0
            .add(ResourceType {
                kind: Default::default(),
                container: self.type_container,
            });
        // Update container with first entry
        self.containers[self.type_container].first = self.meta_type.to_handle();
        // Update entry with type and data slot (itself)
        self.entries[self.meta_type.0 .0].slot = meta_type_data_slot;
        self.entries[self.meta_type.0 .0].ty = self.meta_type;
    }

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        Ok(())
    }

    fn get_type(&self, ty: ResourceTypeHandle) -> Option<&ResourceType> {
        self.containers[self.type_container]
            .container
            .as_any()
            .downcast_ref::<NativeResourceContainer<ResourceType>>()
            .unwrap()
            .0
            .get(self.entries[ty.0 .0].slot)
    }

    fn create_container(&mut self, ty: ResourceTypeHandle) -> SlotId {
        let container = self.get_type(ty).unwrap().create_container();
        let container = self.containers.add(ContainerEntry {
            container,
            first: ResourceHandle::null(),
        });
        // Save container id
        self.containers
            .get_mut(self.type_container)
            .unwrap()
            .container
            .as_any_mut()
            .downcast_mut::<NativeResourceContainer<ResourceType>>()
            .unwrap()
            .0
            .get_mut(ty.to_handle().0)
            .unwrap()
            .container = container;
        container
    }

    pub(crate) fn add<R: Resource>(
        &mut self,
        data: R,
        ty: ResourceTypeHandle,
        owner: ActivityId,
        key: Option<&str>,
    ) -> Result<ResourceHandle, ResourceError> {
        // Allocate container if missing
        if self.get_type(ty).is_none() {
            self.create_container(ty);
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
        let container = self.get_type(ty).unwrap().container;
        // Create new entry
        let mut entry = ResourceEntry {
            key,
            ty,
            slot: SlotId::null(),
            owner,
            ref_count: 0,
            prev: ResourceHandle::null(),
            next: ResourceHandle::null(),
        };
        // Append entry to container
        entry.next = self.containers[container].first;
        let handle: ResourceHandle = ResourceHandle(self.entries.add(entry));
        self.containers[container].first = handle;
        // Load asset
        // TODO: preload resource in container ? wait for read ? define proper strategy
        self.entries[handle.0].slot = self
            .containers
            .get_mut(container)
            .unwrap()
            .container
            .as_any_mut()
            .downcast_mut::<NativeResourceContainer<R>>()
            .expect("Invalid native resource container")
            .0
            .add(data);
        Ok(handle)
    }

    pub(crate) fn add_resource_type(
        &mut self,
        data: ResourceType,
        owner: ActivityId,
        key: Option<&str>,
    ) -> Result<ResourceTypeHandle, ResourceError> {
        self.add(data, self.meta_type, owner, key)
            .map(|handle| handle.into())
    }

    pub(crate) fn get<R: Resource>(
        &self,
        handle: impl ToResourceHandle,
    ) -> Result<&R, ResourceError> {
        // Find entry
        let entry = self
            .entries
            .get(handle.to_handle().0)
            .ok_or(ResourceError::ResourceNotFound)?;
        // Find slot
        let slot = entry.slot;
        if slot.is_null() {
            return Err(ResourceError::ResourceNotLoaded);
        }
        // Find container
        let container = self
            .get_type(entry.ty)
            .ok_or(ResourceError::ResourceTypeNotFound)?
            .container;
        // Read resource
        Ok(self
            .containers
            .get(container)
            .unwrap()
            .container
            .as_any()
            .downcast_ref::<NativeResourceContainer<R>>()
            .expect("Invalid native resource container")
            .0
            .get(slot)
            .unwrap())
    }

    pub(crate) fn get_mut<R: Resource>(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> Result<&mut R, ResourceError> {
        // Find entry
        let entry = self
            .entries
            .get(handle.to_handle().0)
            .ok_or(ResourceError::ResourceNotFound)?;
        // Find slot
        let slot = entry.slot;
        if slot.is_null() {
            return Err(ResourceError::ResourceNotLoaded);
        }
        // Find container
        let container = self
            .get_type(entry.ty)
            .ok_or(ResourceError::ResourceTypeNotFound)?
            .container;
        // Read resource
        Ok(self
            .containers
            .get_mut(container)
            .unwrap()
            .container
            .as_any_mut()
            .downcast_mut::<NativeResourceContainer<R>>()
            .expect("Invalid native resource container")
            .0
            .get_mut(slot)
            .unwrap())
    }

    pub(crate) fn get_unchecked<R: Resource>(&self, handle: impl ToResourceHandle) -> &R {
        let (ty, slot) = {
            let entry = &self.entries[handle.to_handle().0];
            (entry.ty, entry.slot)
        };
        let container = self.get_type(ty).unwrap().container;
        &self.containers[container]
            .container
            .as_any()
            .downcast_ref::<NativeResourceContainer<R>>()
            .unwrap()
            .0[slot]
    }

    pub(crate) fn get_mut_unchecked<R: Resource>(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> &mut R {
        let (ty, slot) = {
            let entry = &self.entries[handle.to_handle().0];
            (entry.ty, entry.slot)
        };
        let container = self.get_type(ty).unwrap().container;
        &mut self.containers[container]
            .container
            .as_any_mut()
            .downcast_mut::<NativeResourceContainer<R>>()
            .unwrap()
            .0[slot]
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = ResourceHandle> + '_ {
        self.entries.keys().map(|id| ResourceHandle(id))
    }

    pub(crate) fn iter_typed(&self, ty: ResourceTypeHandle) -> TypedResourceIterator<'_> {
        if let Some(ty) = self.get_type(ty) {
            let container = ty.container;
            TypedResourceIterator::new(&self.entries, self.containers[container].first)
        } else {
            TypedResourceIterator::new(&self.entries, ResourceHandle::null())
        }
    }

    pub(crate) fn iter_native_mut<R: Resource>(
        &mut self,
        ty: ResourceTypeHandle,
    ) -> TypedNativeResourceIteratorMut<'_, R> {
        if let Some(ty) = self.get_type(ty) {
            let container = ty.container;
            TypedNativeResourceIteratorMut::new(
                &self.entries,
                Some(
                    self.containers[container]
                        .container
                        .as_any_mut()
                        .downcast_mut()
                        .unwrap(),
                ),
                self.containers[container].first,
            )
        } else {
            TypedNativeResourceIteratorMut::new(&self.entries, None, ResourceHandle::null())
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
                return Some(ResourceHandle(id));
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
                let entry = &self.entries[handle.0];
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
        let id = handle.to_handle().0;
        self.entries
            .get(id)
            .map(|entry| ResourceInfo {
                key: entry.key.as_str(),
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
        let id = handle.to_handle().0;
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
        let id = handle.to_handle().0;
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
