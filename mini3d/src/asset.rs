use core::result::Result;

use crate::io::IOManager;
use crate::registry::asset::{AssetReferenceTrait, AssetRegistry, AssetType, AssetTypeTrait};
use crate::registry::component::ComponentRegistry;
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};
use crate::utils::slotmap::{DenseSlotMap, SlotId, SparseSecondaryMap};
use crate::utils::uid::ToUID;

use self::container::{
    AnyAssetContainer, PrivateAnyAssetContainerMut, PrivateAnyAssetContainerRef,
};
use self::error::AssetError;
use self::handle::AssetHandle;
use self::key::AssetKey;

pub mod container;
pub mod error;
pub mod handle;
pub mod key;

pub(crate) enum AssetSource {
    Persistent,
    IO,
}

pub struct AssetInfo<'a> {
    pub path: &'a str,
}

struct AssetEntry {
    key: AssetKey,
    ty: AssetType,
    slot: SlotId, // Null if not loaded
    source: AssetSource,
}

#[derive(Default)]
pub struct AssetManager {
    containers: SparseSecondaryMap<Box<dyn AnyAssetContainer>>, // AssetType -> Container
    entries: DenseSlotMap<AssetEntry>,                          // AssetType -> AssetEntry
}

impl AssetManager {
    pub(crate) fn save_state(
        &self,
        registry: &ComponentRegistry,
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
        registry: &ComponentRegistry,
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

        // // Check that all assets have a default value
        // for (asset, _) in self.defaults.iter() {
        //     if self.containers.get(asset).is_none() {
        //         return Err(DecoderError::CorruptedData);
        //     }
        // }
        Ok(())
    }

    pub(crate) fn on_registry_update(&mut self, registry: &AssetRegistry) {
        for (id, entry) in registry.entries.iter() {
            if !self.containers.contains(id) {
                let container = entry.reflection.create_asset_container();
                self.containers.insert(id, container);
            }
        }
    }

    pub fn persist<T: AssetTypeTrait>(
        &mut self,
        ty: T,
        key: &str,
        data: T::Data,
    ) -> Result<AssetHandle, AssetError> {
        if self.find(key).is_some() {
            return Err(AssetError::DuplicatedAssetEntry);
        }
        let id = self.entries.add(AssetEntry {
            key: AssetKey::new(key),
            ty: AssetType(ty.id()),
            slot: SlotId::null(),
            source: AssetSource::Persistent,
        });
        // TODO: preload asset in container ? wait for read ? define proper strategy
        if let Some(container) = self.containers.get_mut(ty.id()) {
            self.entries[id].slot =
                T::insert_container(PrivateAnyAssetContainerMut(container.as_mut()), data);
            Ok(AssetHandle {
                id,
                key: key.to_uid(),
            })
        } else {
            // TODO: report proper error (not sync with registry ?)
            Err(AssetError::AssetTypeNotFound)
        }
    }

    pub fn remove(&mut self, handle: AssetHandle) -> Result<(), AssetError> {
        if !self.entries.contains(handle.id) {
            return Err(AssetError::AssetNotFound);
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

    pub fn load<T: AssetTypeTrait>(
        &mut self,
        io: &mut IOManager,
        handle: AssetHandle,
    ) -> Result<T::Ref<'_>, AssetError> {
        let entry = self
            .entries
            .get(handle.id)
            .ok_or(AssetError::AssetNotFound)?;
        if !entry.slot.is_null() {
            Ok(T::asset_ref(
                PrivateAnyAssetContainerRef(self.containers[entry.ty.0].as_ref()),
                entry.slot,
            ))
        } else {
            todo!("Load asset from source")
        }
    }

    pub fn read<T: AssetReferenceTrait>(
        &self,
        handle: AssetHandle,
    ) -> Result<<T::AssetType as AssetTypeTrait>::Ref<'_>, AssetError> {
        let entry = self
            .entries
            .get(handle.id)
            .ok_or(AssetError::AssetNotFound)?;
        if !entry.slot.is_null() {
            Ok(<T::AssetType as AssetTypeTrait>::asset_ref(
                PrivateAnyAssetContainerRef(self.containers[entry.ty.0].as_ref()),
                entry.slot,
            ))
        } else {
            Err(AssetError::AssetNotLoaded)
        }
    }

    pub fn find(&self, key: impl ToUID) -> Option<AssetHandle> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.key.to_uid() == key.to_uid())
            .map(|(id, entry)| AssetHandle {
                id,
                key: entry.key.to_uid(),
            })
    }

    pub fn info(&self, handle: AssetHandle) -> Result<AssetInfo, AssetError> {
        self.entries
            .get(handle.id)
            .map(|entry| AssetInfo {
                path: entry.key.as_str(),
            })
            .ok_or(AssetError::AssetNotFound)
    }
}
