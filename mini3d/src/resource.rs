use crate::activity::ActivityId;
use crate::feature::core;
use crate::feature::core::component::{Component, ComponentHandle};
use crate::feature::core::resource::{Resource, ResourceData};
use crate::feature::core::system::System;
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};
use crate::utils::prng::PCG32;
use crate::utils::slotmap::{SlotId, SlotMap};
use crate::utils::uid::ToUID;

use self::container::{NativeResourceContainer, ResourceContainer};
use self::error::ResourceError;
use self::handle::{ResourceHandle, ResourceTypeHandle, ToResourceHandle};
use self::key::ResourceKey;

pub mod container;
pub mod error;
pub mod handle;
pub mod key;

pub struct ResourceInfo<'a> {
    pub path: &'a str,
}

struct ContainerEntry {
    container: Box<dyn ResourceContainer>,
    first: SlotId, // First entry
}

struct ResourceEntry {
    key: ResourceKey,
    ty: ResourceTypeHandle,
    owner: ActivityId,
    ref_count: usize,
    slot: SlotId, // Null if not loaded
    prev: SlotId, // Previous entry of same type
    next: SlotId, // Next entry of same type
}

pub struct ResourceManager {
    containers: SlotMap<ContainerEntry>,
    entries: SlotMap<ResourceEntry>,
    type_container: SlotId, // Container with list of types
    resource_type_handle: ResourceTypeHandle,
    prng: PCG32,
}

impl ResourceManager {
    pub(crate) fn new(root: ActivityId) -> Self {
        let mut manager = Self {
            containers: Default::default(),
            entries: Default::default(),
            type_container: SlotId::null(),
            resource_type_handle: ResourceTypeHandle(SlotId::null()),
            prng: PCG32::new(1234),
        };
        // Add resource type container
        manager.type_container = manager
            .containers
            .add(Box::new(NativeResourceContainer::<Resource>::default()));
        // As all container entry must have a resource type, we add a dummy resource type
        manager.resource_type_handle = ResourceTypeHandle(manager.entries.add(ResourceEntry {
            key: "_resource_type",
            ty: Default::default(),
            owner: root,
            ref_count: 0,
            slot: SlotId::null(),
            prev: SlotId::null(),
            next: SlotId::null(),
        }));
        // Define core resources
        macro_rules! define_resource {
            ($resource: ty) => {
                manager
                    .define_resource(<$resource>::NAME, Resource::native::<$resource>(), root)
                    .unwrap()
            };
        }
        manager.types.component = define_resource!(core::component::Component);
        manager.types.system = define_resource!(core::system::System);
        define_resource!(core::system::SystemStage);
        define_resource!(core::system::SystemSet);
        define_resource!(core::activity::ActivityDescriptor);
        define_resource!(core::structure::StructDefinition);
        manager
    }
}

impl ResourceManager {
    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        Ok(())
    }

    pub(crate) fn add<R: ResourceData>(
        &mut self,
        ty: impl ToResourceHandle,
        key: Option<&str>,
        owner: ActivityId,
        data: R,
    ) -> Result<ResourceHandle, ResourceError> {
        // Allocate container if missing
        if self.get::<Resource>(ty).unwrap().container_id.is_null() {
            let container = self.get::<Resource>(ty).unwrap().create_container();
            let container_id = self.containers.add(ContainerEntry {
                container,
                first: SlotId::null(),
            });
            // Setup container ID
            self.containers
                .get_mut(self.type_container)
                .unwrap()
                .container
                .as_any_mut()
                .downcast_mut::<NativeResourceContainer<Resource>>()
                .unwrap()
                .0
                .get_mut(ty.to_handle().0)
                .unwrap()
                .container_id = container_id;
        }
        // Check duplicated entry
        let key = if let Some(key) = key {
            if self.find(ty, key).is_some() {
                return Err(ResourceError::DuplicatedAssetEntry);
            }
            ResourceKey::new(key)
        } else {
            // Generate random key
            ResourceKey::random(&mut self.prng)
        };
        // Create new entry
        let mut entry = ResourceEntry {
            key,
            ty: ty.to_handle(),
            slot: SlotId::null(),
            owner,
            ref_count: 0,
            prev: SlotId::null(),
            next: SlotId::null(),
        };
        // Increment resource type
        self.increment_ref(ty.to_handle())
            .expect("Failed to increment resource type ref count");
        // Append entry to container
        let resource_ty = self.get::<Resource>(ty).unwrap();
        let container_id = resource_ty.container_id;
        entry.next = self.containers[container_id].first;
        let id = self.entries.add(entry);
        self.containers[container_id].first = id;
        // Load asset
        // TODO: preload resource in container ? wait for read ? define proper strategy
        self.entries[id].slot = self
            .containers
            .get_mut(container_id)
            .unwrap()
            .container
            .as_any_mut()
            .downcast_mut::<NativeResourceContainer<R>>()
            .expect("Invalid native resource container")
            .insert(data);
        Ok(ResourceHandle(id))
    }

    // pub(crate) fn load<R: Resource>(
    //     &mut self,
    //     io: &mut IOManager,
    //     handle: impl ToResourceHandle,
    // ) -> Result<&R, ResourceError> {
    //     let id = handle.to_handle().0;
    //     let entry = self
    //         .entries
    //         .get(id)
    //         .ok_or(ResourceError::ResourceNotFound)?;
    //     if !entry.slot.is_null() {
    //         Ok(T::resource_ref(
    //             PrivateResourceContainerRef(self.containers[entry.ty.0].as_ref()),
    //             entry.slot,
    //         ))
    //     } else {
    //         todo!("Load resource from source")
    //     }
    // }

    pub(crate) fn get<R: ResourceData>(
        &self,
        handle: impl ToResourceHandle,
    ) -> Result<&R, ResourceError> {
        // Find entry
        let id = handle.to_handle().0;
        let entry = self
            .entries
            .get(id)
            .ok_or(ResourceError::ResourceNotFound)?;
        // Read resource
        if !entry.slot.is_null() {
            Ok(self
                .containers
                .get(entry.ty.handle().0)
                .ok_or(ResourceError::ResourceTypeNotFound)?
                .as_any()
                .downcast_ref::<NativeResourceContainer<R>>()
                .expect("Invalid native resource container")
                .get(entry.slot)
                .expect("native static resource slot"))
        } else {
            Err(ResourceError::ResourceNotLoaded)
        }
    }

    pub(crate) fn get_mut<R: ResourceData>(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> Result<&mut R, ResourceError> {
        // Find entry
        let id = handle.to_handle().0;
        let entry = self
            .entries
            .get(id)
            .ok_or(ResourceError::ResourceNotFound)?;
        // Read resource
        if !entry.slot.is_null() {
            Ok(self
                .containers
                .get_mut(entry.ty.handle().0)
                .ok_or(ResourceError::ResourceTypeNotFound)?
                .as_any_mut()
                .downcast_mut::<NativeResourceContainer<R>>()
                .expect("Invalid native resource container")
                .get_mut(entry.slot)
                .expect("native static resource slot"))
        } else {
            Err(ResourceError::ResourceNotLoaded)
        }
    }

    pub(crate) fn iter_mut<R: ResourceData>(
        &mut self,
        ty: ResourceTypeHandle,
    ) -> impl Iterator<Item = &mut R> {
        let container_id = self.get::<Resource>(ty).unwrap().container_id;
        self.containers
            .get_mut(container_id)
            .unwrap()
            .container
            .as_any_mut()
            .downcast_mut::<NativeResourceContainer<R>>()
            .unwrap()
            .0
            .iter_mut()
    }

    pub(crate) fn get_mut_unchecked<R: ResourceData>(
        &mut self,
        ty: ResourceTypeHandle,
        handle: ResourceHandle,
    ) -> &mut R {
        let container_id = self.entries[handle.0].ty.0 .0;
        &mut self.containers[container_id]
            .container
            .as_any_mut()
            .downcast_mut::<NativeResourceContainer<R>>()
            .unwrap()
            .0[self.entries[handle.0].slot]
    }

    pub(crate) fn find(
        &self,
        ty: impl ToResourceHandle,
        key: impl ToUID,
    ) -> Option<ResourceHandle> {
        let mut first = self
            .containers
            .get(ty.to_handle().0)
            .map(|container_entry| container_entry.first)
            .unwrap_or_default();
        while !first.is_null() {
            let entry = self.entries.get(first).unwrap();
            if entry.key.to_uid() == key.to_uid() {
                return Some(ResourceHandle(first));
            }
            first = entry.next;
        }
        None
    }

    pub(crate) fn find_type(&self, key: impl ToUID) -> Option<ResourceTypeHandle> {
        self.find(self.resource_type_handle, key)
    }

    pub(crate) fn define_resource(
        &mut self,
        name: &str,
        ty: Resource,
        owner: ActivityId,
    ) -> Result<ResourceTypeHandle, ResourceError> {
        Ok(self.add(self.resource_type_handle, Some(name), owner, ty)?)
    }

    pub(crate) fn define_component(
        &mut self,
        name: &str,
        component: Component,
        owner: ActivityId,
    ) -> Result<ComponentHandle, ResourceError> {
        Ok(self.add(self.component_type_handle, Some(name), owner, component)?)
    }

    pub(crate) fn define_system(
        &mut self,
        name: &str,
        system: System,
        owner: ActivityId,
    ) -> Result<ResourceTypeHandle, ResourceError> {
        Ok(self.add(self.system_type_handle, Some(name), owner, system)?)
    }

    pub(crate) fn info(
        &self,
        handle: impl ToResourceHandle,
    ) -> Result<ResourceInfo, ResourceError> {
        let id = handle.to_handle().0;
        self.entries
            .get(id)
            .map(|entry| ResourceInfo {
                path: entry.key.as_str(),
            })
            .ok_or(ResourceError::ResourceNotFound)
    }

    pub(crate) fn increment_ref(&mut self, handle: ResourceHandle) -> Result<(), ResourceError> {
        let id = handle.to_handle().0;
        let entry = self
            .entries
            .get_mut(id)
            .ok_or(ResourceError::ResourceNotFound)?;
        entry.ref_count += 1;
        Ok(())
    }

    pub(crate) fn decrement_ref(&mut self, handle: ResourceHandle) -> Result<(), ResourceError> {
        let id = handle.0;
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
