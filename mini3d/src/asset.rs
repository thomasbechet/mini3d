use core::result::Result;

use crate::registry::asset::{AssetRegistry, AssetType, AssetTypeHandle};
use crate::registry::component::ComponentRegistry;
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};
use crate::utils::slotmap::{DenseSlotMap, SlotId, SparseSecondaryMap};
use crate::utils::string::AsciiArray;

use self::container::AnyAssetContainer;
use self::error::AssetError;
use self::handle::{
    AssetBundle, AssetHandle, PrivateAnyAssetContainerMut, PrivateAnyAssetContainerRef,
};

pub mod container;
pub mod error;
pub mod handle;
pub mod reference;

type AssetEntryId = SlotId;

pub(crate) enum AssetSource {
    Persistent,
    IO,
}

pub struct AssetInfo<'a> {
    pub name: &'a str,
}

pub(crate) const MAX_ASSET_NAME_LEN: usize = 64;

struct AssetEntry {
    name: AsciiArray<MAX_ASSET_NAME_LEN>,
    ty: AssetType,
    slot: SlotId, // Null if not loaded
    source: AssetSource,
    bundle: AssetBundle,
    next_in_bundle: AssetEntryId,
    prev_in_bundle: AssetEntryId,
}

pub(crate) const MAX_ASSET_BUNDLE_NAME_LEN: usize = 64;

struct AssetBundleEntry {
    name: AsciiArray<MAX_ASSET_BUNDLE_NAME_LEN>,
    first_entry: AssetEntryId, // Null if empty
}

pub struct AssetManager {
    containers: SparseSecondaryMap<Box<dyn AnyAssetContainer>>, // AssetType -> Container
    bundles: DenseSlotMap<AssetBundleEntry>,                    // AssetBundleId -> AssetBundle
    entries: DenseSlotMap<AssetEntry>,                          // AssetType -> AssetEntry
}

impl Default for AssetManager {
    fn default() -> Self {
        let mut manager = Self {
            containers: Default::default(),
            bundles: Default::default(),
            entries: Default::default(),
        };
        manager.add_bundle(AssetBundle::DEFAULT).unwrap();
        manager
    }
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

    fn add_entry(
        &mut self,
        name: &str,
        ty: AssetType,
        bundle: AssetBundle,
        source: AssetSource,
    ) -> Result<SlotId, AssetError> {
        let id = bundle.0;
        if let Some(bundle_entry) = self.bundles.get_mut(id) {
            let slot = self.entries.add(AssetEntry {
                name: name.into(),
                ty,
                slot: SlotId::null(),
                source,
                bundle,
                next_in_bundle: SlotId::null(),
                prev_in_bundle: SlotId::null(),
            });
            // Update chain list
            if !bundle_entry.first_entry.is_null() {
                self.entries[bundle_entry.first_entry].prev_in_bundle = slot;
                self.entries[slot].next_in_bundle = bundle_entry.first_entry;
            }
            bundle_entry.first_entry = slot;
            Ok(slot)
        } else {
            Err(AssetError::BundleNotFound)
        }
    }

    fn remove_entry(&mut self, slot: SlotId) {
        let bundle = self.entries[slot].bundle;
        // Remove from chain
        let next = self.entries[slot].next_in_bundle;
        let prev = self.entries[slot].prev_in_bundle;
        if prev.is_null() {
            self.bundles[bundle.0].first_entry = next;
        } else {
            self.entries[prev].next_in_bundle = next;
        }
        if !next.is_null() {
            self.entries[next].prev_in_bundle = prev;
        }
        // Remove entry
        self.entries.remove(slot);
    }

    pub fn add<H: AssetHandle>(
        &mut self,
        ty: <H as AssetHandle>::TypeHandle,
        name: &str,
        bundle: AssetBundle,
        data: <H::TypeHandle as AssetTypeHandle>::Data,
    ) -> Result<H, AssetError> {
        if self.find::<H>(name).is_some() {
            return Err(AssetError::DuplicatedAssetEntry);
        }
        let id = self.add_entry(name, AssetType(ty.id()), bundle, AssetSource::IO)?;
        // TODO: preload asset in container ? wait for read ? define proper strategy
        if let Some(container) = self.containers.get_mut(ty.id()) {
            self.entries[id].slot = <H::TypeHandle as AssetTypeHandle>::insert_container(
                PrivateAnyAssetContainerMut(container.as_mut()),
                data,
            );
            Ok(<H>::new(id))
        } else {
            // TODO: report proper error (not sync with registry ?)
            Err(AssetError::AssetTypeNotFound)
        }
    }

    pub fn remove<H: AssetTypeHandle>(&mut self, handle: H) -> Result<(), AssetError> {
        let id = handle.id();
        if !self.entries.contains(id) {
            return Err(AssetError::AssetNotFound);
        }
        // TODO: remove cached data from container
        if !self.entries[id].slot.is_null() {
            <H>::remove_container(
                PrivateAnyAssetContainerMut(self.containers.get_mut(id).unwrap().as_mut()),
                self.entries[id].slot,
            );
        }
        // Remove entry
        self.remove_entry(id);
        Ok(())
    }

    pub fn find<H: AssetHandle>(&self, name: &str) -> Option<H> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.name.as_str() == name)
            .filter(|(_, entry)| {
                <H::TypeHandle>::check_type(PrivateAnyAssetContainerRef(
                    self.containers.get(entry.ty.0).unwrap().as_ref(),
                ))
            })
            .map(|(id, _)| H::new(id))
    }

    pub fn info<H: AssetHandle>(&self, handle: H) -> Result<AssetInfo, AssetError> {
        let id = handle.id();
        self.entries
            .get(id)
            .map(|entry| AssetInfo { name: &entry.name })
            .ok_or(AssetError::AssetNotFound)
    }

    pub fn read<H: AssetHandle>(&self, handle: H) -> Result<H::Ref<'_>, AssetError> {
        let id = handle.id();
        let entry = self.entries.get(id).ok_or(AssetError::AssetNotFound)?;
        if !entry.slot.is_null() {
            Ok(handle.asset_ref(
                entry.slot,
                PrivateAnyAssetContainerRef(self.containers[entry.ty.0].as_ref()),
            ))
        } else {
            Err(AssetError::AssetNotFound) // TODO: load the asset from source
        }
    }

    pub fn write<H: AssetHandle>(&self, handle: H, asset: H::Ref<'_>) -> Result<(), AssetError> {
        Ok(())
    }

    pub fn add_bundle(&mut self, name: &str) -> Result<AssetBundle, AssetError> {
        if self
            .bundles
            .values()
            .any(|entry| entry.name.as_str() == name)
        {
            return Err(AssetError::DuplicatedBundle);
        }
        let slot = self.bundles.add(AssetBundleEntry {
            name: name.into(),
            first_entry: SlotId::null(),
        });
        Ok(AssetBundle(slot))
    }

    pub fn remove_bundle(&mut self, bundle: AssetBundle) -> Result<(), AssetError> {
        let id = bundle.0;
        if !self.bundles.contains(id) {
            return Err(AssetError::BundleNotFound);
        }
        // Remove all entries
        while !self.bundles[id].first_entry.is_null() {
            self.remove_entry(self.bundles[id].first_entry);
        }
        // Remove bundle
        self.bundles.remove(id);
        Ok(())
    }

    pub fn find_bundle(&self, name: &str) -> Option<AssetBundle> {
        self.bundles
            .iter()
            .find(|(_, entry)| entry.name.as_str() == name)
            .map(|(id, _)| AssetBundle(id))
    }
}
