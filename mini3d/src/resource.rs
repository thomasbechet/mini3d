use core::result::Result;

use crate::io::IOManager;
use crate::program::ProgramId;
use crate::registry::component::ComponentRegistryManager;
use crate::registry::resource::{Resource, ResourceRegistryManager, ResourceType};
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};
use crate::utils::slotmap::{DenseSlotMap, SlotId, SparseSecondaryMap};
use crate::utils::uid::ToUID;

use self::container::{
    NativeResourceContainer, PrivateResourceContainerMut, PrivateResourceContainerRef,
    ResourceContainer,
};
use self::error::ResourceError;
use self::handle::ResourceHandle;
use self::key::ResourceKey;

pub mod container;
pub mod error;
pub mod handle;
pub mod key;

pub enum ResourceSharingMode {}

pub struct ResourceInfo<'a> {
    pub path: &'a str,
}

struct ResourceEntry {
    key: ResourceKey,
    ty: ResourceType,
    owner: ProgramId,
    ref_count: usize,
    slot: SlotId, // Null if not loaded
}

#[derive(Default)]
pub struct ResourceManager {
    containers: SparseSecondaryMap<Box<dyn ResourceContainer>>,
    entries: DenseSlotMap<ResourceEntry>,
}

impl ResourceManager {
    pub(crate) fn save_state(
        &self,
        registry: &ComponentRegistryManager,
        encoder: &mut impl Encoder,
    ) -> Result<(), EncoderError> {
        // encoder.write_u32(self.bundles.len() as u32)?;
        // for uid in self.bundles.keys() {
        //     self.serialize_bundle(*uid, registry, encoder)
        //         .map_err(|_| EncoderError::Unsupported)?;
        // }
        Ok(())
    }

    pub(crate) fn load_state(
        &mut self,
        registry: &ComponentRegistryManager,
        decoder: &mut impl Decoder,
    ) -> Result<(), DecoderError> {
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

    pub(crate) fn on_registry_update(&mut self, registry: &ResourceRegistryManager) {
        for (id, entry) in registry.entries.iter() {
            if !self.containers.contains(id) {
                let container = entry.reflection.create_resource_container();
                self.containers.insert(id, container);
            }
        }
    }

    pub(crate) fn persist<T: ResourceTypeTrait>(
        &mut self,
        ty: T,
        key: &str,
        data: T::Data,
    ) -> Result<ResourceHandle, ResourceError> {
        if self.find(key).is_some() {
            return Err(ResourceError::DuplicatedAssetEntry);
        }
        let id = self.entries.add(ResourceEntry {
            key: ResourceKey::new(key),
            ty: ResourceType(ty.id()),
            slot: SlotId::null(),
            source: AssetSource::Persistent,
        });
        // TODO: preload resource in container ? wait for read ? define proper strategy
        if let Some(container) = self.containers.get_mut(ty.id()) {
            self.entries[id].slot =
                T::insert_container(PrivateResourceContainerMut(container.as_mut()), data);
            Ok(ResourceHandle {
                id,
                uid: key.to_uid(),
            })
        } else {
            // TODO: report proper error (not sync with registry ?)
            Err(ResourceError::ResourceTypeNotFound)
        }
    }

    pub(crate) fn remove(&mut self, handle: ResourceHandle) -> Result<(), ResourceError> {
        if !self.entries.contains(handle.id) {
            return Err(ResourceError::ResourceNotFound);
        }
        // TODO: remove cached data from container
        if !self.entries[handle.id].slot.is_null() {
            self.containers
                .get_mut(handle.id)
                .unwrap()
                .remove(self.entries[handle.id].slot);
        }
        // Remove entry
        self.entries.remove(handle.id);
        Ok(())
    }

    pub(crate) fn load<R: Resource>(
        &mut self,
        io: &mut IOManager,
        handle: ResourceHandle,
    ) -> Result<&R, ResourceError> {
        let entry = self
            .entries
            .get(handle.id)
            .ok_or(ResourceError::ResourceNotFound)?;
        if !entry.slot.is_null() {
            Ok(T::resource_ref(
                PrivateResourceContainerRef(self.containers[entry.ty.0].as_ref()),
                entry.slot,
            ))
        } else {
            todo!("Load resource from source")
        }
    }

    pub(crate) fn read<R: Resource>(&self, handle: ResourceHandle) -> Result<&R, ResourceError> {
        let entry = self
            .entries
            .get(handle.id)
            .ok_or(ResourceError::ResourceNotFound)?;
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
            .map(|(id, entry)| ResourceHandle {
                id,
                uid: entry.key.to_uid(),
            })
    }

    pub(crate) fn info(&self, handle: ResourceHandle) -> Result<ResourceInfo, ResourceError> {
        self.entries
            .get(handle.id)
            .map(|entry| ResourceInfo {
                path: entry.key.as_str(),
            })
            .ok_or(ResourceError::ResourceNotFound)
    }
}
