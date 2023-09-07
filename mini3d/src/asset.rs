use core::result::Result;

use crate::registry::component::{ComponentHandle, ComponentId, ComponentRegistry};
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};
use crate::utils::slotmap::{DenseSlotMap, SlotId, SparseSecondaryMap};
use crate::utils::string::AsciiArray;

use self::container::AnyAssetContainer;
use self::error::AssetError;
use self::handle::{
    AssetBundleId, AssetHandle, PrivateAnyAssetContainerMut, PrivateAnyAssetContainerRef,
};

pub mod container;
pub mod error;
pub mod handle;

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
    component: ComponentId,
    slot: SlotId, // Null if not loaded
    source: AssetSource,
    bundle: AssetBundleId,
    next_in_bundle: AssetEntryId,
    prev_in_bundle: AssetEntryId,
}

pub struct AssetBundle;

impl AssetBundle {
    pub const DEFAULT: &'static str = "default";
}

pub(crate) const MAX_ASSET_BUNDLE_NAME_LEN: usize = 64;

struct AssetBundleEntry {
    name: AsciiArray<MAX_ASSET_BUNDLE_NAME_LEN>,
    first_entry: AssetEntryId, // Null if empty
}

pub struct AssetManager {
    containers: SparseSecondaryMap<Box<dyn AnyAssetContainer>>, // ComponentId -> Container
    bundles: DenseSlotMap<AssetBundleEntry>,                    // AssetBundleId -> AssetBundle
    entries: DenseSlotMap<AssetEntry>,                          // AssetId -> AssetEntry
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

    pub(crate) fn preallocate<H: ComponentHandle>(
        &mut self,
        handle: H,
        registry: &ComponentRegistry,
    ) {
        let id = handle.id();
        if !self.containers.contains(id.into()) {
            let container = registry
                .definition(handle)
                .unwrap()
                .reflection
                .create_asset_container();
            self.containers.insert(id.into(), container);
        }
    }

    fn add_entry(
        &mut self,
        name: &str,
        component: ComponentId,
        bundle: AssetBundleId,
        source: AssetSource,
    ) -> Result<SlotId, AssetError> {
        let id = bundle.id();
        if let Some(bundle_entry) = self.bundles.get_mut(id) {
            let slot = self.entries.add(AssetEntry {
                name: name.into(),
                component,
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
            self.bundles[bundle.id()].first_entry = next;
        } else {
            self.entries[prev].next_in_bundle = next;
        }
        if !next.is_null() {
            self.entries[next].prev_in_bundle = prev;
        }
        // Remove entry
        self.entries.remove(slot);
    }

    pub(crate) fn add<C: ComponentHandle>(
        &mut self,
        handle: C,
        name: &str,
        bundle: AssetBundleId,
        source: AssetSource,
        data: <C::AssetHandle as AssetHandle>::Data,
    ) -> Result<C::AssetHandle, AssetError> {
        if self.find::<C::AssetHandle>(name).is_some() {
            return Err(AssetError::DuplicatedAssetEntry);
        }
        let id = self.add_entry(name, handle.id(), bundle, source)?;
        // TODO: preload asset in container ? wait for read ? define proper strategy
        if let Some(container) = self.containers.get_mut(handle.id().into()) {
            self.entries[id].slot = <C::AssetHandle as AssetHandle>::insert_container(
                PrivateAnyAssetContainerMut(container.as_mut()),
                data,
            );
            Ok(<C::AssetHandle as AssetHandle>::new(id))
        } else {
            // TODO: report proper error (not sync with registry ?)
            Err(AssetError::AssetTypeNotFound)
        }
    }

    pub(crate) fn remove<H: AssetHandle>(&mut self, handle: H) -> Result<(), AssetError> {
        let id = handle.id();
        if !self.entries.contains(id) {
            return Err(AssetError::AssetNotFound);
        }
        // TODO: remove cached data from container
        if !self.entries[id].slot.is_null() {
            <H as AssetHandle>::remove_container(
                PrivateAnyAssetContainerMut(self.containers.get_mut(id.into()).unwrap().as_mut()),
                self.entries[id].slot,
            );
        }
        // Remove entry
        self.remove_entry(id);
        Ok(())
    }

    pub(crate) fn find<H: AssetHandle>(&self, name: &str) -> Option<H> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.name.as_str() == name)
            .filter(|(_, entry)| {
                H::check_type(PrivateAnyAssetContainerRef(
                    self.containers
                        .get(entry.component.into())
                        .unwrap()
                        .as_ref(),
                ))
            })
            .map(|(id, _)| H::new(id))
    }

    pub(crate) fn info<H: AssetHandle>(&self, handle: H) -> Result<AssetInfo, AssetError> {
        let id = handle.id();
        self.entries
            .get(id)
            .map(|entry| AssetInfo { name: &entry.name })
            .ok_or(AssetError::AssetNotFound)
    }

    pub(crate) fn read<H: AssetHandle>(&self, handle: H) -> Result<H::AssetRef<'_>, AssetError> {
        let slot = handle.id();
        let entry = self.entries.get(slot).ok_or(AssetError::AssetNotFound)?;
        if !entry.slot.is_null() {
            Ok(handle.asset_ref(PrivateAnyAssetContainerRef(
                self.containers[entry.component.into()].as_ref(),
            )))
        } else {
            Err(AssetError::AssetNotFound) // TODO: load the asset from source
        }
    }

    pub(crate) fn write<H: AssetHandle>(
        &self,
        handle: H,
        asset: H::AssetRef<'_>,
    ) -> Result<(), AssetError> {
        Ok(())
    }

    pub(crate) fn add_bundle(&mut self, name: &str) -> Result<AssetBundleId, AssetError> {
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
        Ok(AssetBundleId::new(slot))
    }

    pub(crate) fn remove_bundle(&mut self, bundle: AssetBundleId) -> Result<(), AssetError> {
        let id = bundle.id();
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
}
