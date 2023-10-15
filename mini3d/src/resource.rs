use core::result::Result;

use crate::activity::ActivityId;
use crate::feature::core::resource_type::{Resource, ResourceType};
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};
use crate::utils::slotmap::{DenseSlotMap, SlotId, SlotMap};
use crate::utils::uid::ToUID;

use self::container::{NativeResourceContainer, ResourceContainer};
use self::error::ResourceError;
use self::handle::{ResourceHandle, ResourceRef, ToResourceHandle};
use self::hook::ResourceAddedHook;
use self::key::ResourceKey;

pub mod container;
pub mod error;
pub mod handle;
pub mod hook;
pub mod key;

pub enum ResourceSharingMode {}

pub struct ResourceInfo<'a> {
    pub path: &'a str,
}

struct ResourceEntry {
    key: ResourceKey,
    ty: ResourceRef,
    owner: ActivityId,
    ref_count: usize,
    slot: SlotId, // Null if not loaded
}

pub struct ResourceManager {
    containers: SlotMap<Box<dyn ResourceContainer>>,
    type_container: SlotId,
    entries: DenseSlotMap<ResourceEntry>,
}

impl ResourceManager {
    pub(crate) fn new(root_activity: ActivityId) -> Self {
        let mut manager = Self {
            containers: Default::default(),
            type_container: SlotId::null(),
            entries: Default::default(),
        };
        // Define core resource type
        manager.type_container = manager
            .containers
            .add(Box::new(NativeResourceContainer::<ResourceType>::default()));
        manager.entries.add(ResourceEntry {
            key: "_ty_resource_type",
            ty: ResourceRef::default(),
            owner: root_activity,
            ref_count: 0,
            slot: SlotId::null(),
        });
        manager
    }
}

impl ResourceManager {
    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        // encoder.write_u32(self.bundles.len() as u32)?;
        // for uid in self.bundles.keys() {
        //     self.serialize_bundle(*uid, registry, encoder)
        //         .map_err(|_| EncoderError::Unsupported)?;
        // }
        Ok(())
    }

    pub(crate) fn load_state(&mut self, decoder: &mut impl Decoder) -> Result<(), DecoderError> {
        // // Clear all data
        // self.bundles.clear();
        // self.containers.clear();
        // self.defaults.clear();

        // // Decode bundles
        // let bundle_count = decoder.read_u32()?;
        // for _ in 0..bundle_count {
        //     let import = ImportAssetBundle::deserialize(registry, decoder)
        //         .map_err(|_| DecoderError::CorruptedData)?;
        //     self.import_bundle(import)
        //         .map_err(|_| DecoderError::CorruptedData)?;
        // }

        // // Decode default values
        // let default_count = decoder.read_u32()?;
        // for _ in 0..default_count {
        //     let uid = UID::deserialize(decoder, &Default::default())?;
        //     if let Some(id) = registry.find_id(uid) {
        //         let default = UID::deserialize(decoder, &Default::default())?;
        //         self.defaults.insert(id.into(), default);
        //     } else {
        //         return Err(DecoderError::CorruptedData);
        //     }
        // }

        // // Check that all resources have a default value
        // for (resource, _) in self.defaults.iter() {
        //     if self.containers.get(resource).is_none() {
        //         return Err(DecoderError::CorruptedData);
        //     }
        // }
        Ok(())
    }

    pub(crate) fn add<R: Resource>(
        &mut self,
        ty: impl ToResourceHandle,
        key: &str,
        owner: ActivityId,
        data: R,
        hook: &mut Option<ResourceAddedHook>,
    ) -> Result<ResourceHandle, ResourceError> {
        // Check duplicated entry
        if self.find(key).is_some() {
            return Err(ResourceError::DuplicatedAssetEntry);
        }
        // Find resource type reference
        let ty_handle = ty.to_handle();
        let ty_reference = self
            .acquire(ty_handle)
            .expect("Invalid resource type reference");
        // Allocate container if missing
        if self
            .read::<ResourceType>(ty_handle)
            .unwrap()
            .container_id
            .is_null()
        {
            let container = self
                .read::<ResourceType>(ty_handle)
                .unwrap()
                .create_container();
            let container_id = self.containers.add(container);
            // Force update resource type
            self.containers
                .get_mut(self.type_container)
                .unwrap()
                .as_any_mut()
                .downcast_mut::<NativeResourceContainer<ResourceType>>()
                .unwrap()
                .0
                .get_mut(ty_reference.id)
                .unwrap()
                .container_id = container_id;
        }
        // Create new entry
        let id = self.entries.add(ResourceEntry {
            key: ResourceKey::new(key),
            ty: ty_reference,
            slot: SlotId::null(),
            owner,
            ref_count: 0,
        });
        let resource_ty = self.read::<ResourceType>(ty_handle).unwrap();
        let container_id = resource_ty.container_id;
        *hook = resource_ty.added_hook;
        // TODO: preload resource in container ? wait for read ? define proper strategy
        self.entries[id].slot = self
            .containers
            .get_mut(container_id)
            .unwrap()
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

    pub(crate) fn read<R: Resource>(
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
                .get(entry.ty.0)
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

    pub(crate) fn find(&self, key: impl ToUID) -> Option<ResourceHandle> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.key.to_uid() == key.to_uid())
            .map(|(id, entry)| ResourceHandle(id))
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

    pub(crate) fn acquire(
        &mut self,
        handle: impl ToResourceHandle,
    ) -> Result<ResourceRef, ResourceError> {
        let id = handle.to_handle().0;
        let entry = self
            .entries
            .get_mut(id)
            .ok_or(ResourceError::ResourceNotFound)?;
        entry.ref_count += 1;
        Ok(ResourceRef {
            id,
            key: entry.key.to_uid(),
        })
    }

    pub(crate) fn release(&mut self, reference: ResourceRef) -> Result<(), ResourceError> {
        let id = reference.id;
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
