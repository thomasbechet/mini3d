use core::result::Result;

use crate::registry::component::{ComponentHandle, ComponentId, ComponentRegistry};
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError};
use crate::utils::generation::{GenerationId, VersionId};
use crate::utils::slotmap::{DenseSlotMap, SlotId, SparseSecondaryMap};

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

struct AssetEntry {
    name: String, // TODO: use a string pool
    component: ComponentId,
    version: VersionId,
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

struct AssetBundleEntry {
    name: String,
    first_entry: AssetEntryId, // Null if empty
    version: VersionId,
}

pub struct AssetManager {
    containers: SparseSecondaryMap<Box<dyn AnyAssetContainer>>, // ComponentId -> Container
    bundles: DenseSlotMap<AssetBundleEntry>,                    // AssetBundleId -> AssetBundle
    entries: DenseSlotMap<AssetEntry>,                          // AssetId -> AssetEntry
    next_version: VersionId,
}

impl Default for AssetManager {
    fn default() -> Self {
        let mut manager = Self {
            containers: Default::default(),
            bundles: Default::default(),
            entries: Default::default(),
            next_version: Default::default(),
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

    fn add_entry(
        &mut self,
        name: &str,
        component: ComponentId,
        bundle: AssetBundleId,
        source: AssetSource,
    ) -> Result<GenerationId, AssetError> {
        let id = bundle.id();
        if let Some(bundle_entry) = self.bundles.get_mut(id.slot()) {
            let version = self.next_version.next();
            let slot = self.entries.add(AssetEntry {
                name: name.to_owned(),
                component,
                version,
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
            Ok(GenerationId::from_slot(slot, version))
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
            self.bundles[bundle.id().slot()].first_entry = next;
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
        registry: &ComponentRegistry,
    ) -> Result<C::AssetHandle, AssetError> {
        if self.find::<C::AssetHandle>(name).is_some() {
            return Err(AssetError::DuplicatedAssetEntry);
        }
        let id = self.add_entry(name, handle.id(), bundle, source)?;
        // Allocate container if needed
        if !self.containers.contains(handle.id().into()) {
            let definition = registry
                .definition(handle)
                .map_err(|_| AssetError::AssetTypeNotFound)?;
            self.containers.insert(
                handle.id().into(),
                definition.reflection.create_asset_container(),
            );
        }
        // TODO: preload asset in container ? wait for read ? define proper strategy
        self.entries[id.slot()].slot = <C::AssetHandle as AssetHandle>::insert_container(
            PrivateAnyAssetContainerMut(
                self.containers
                    .get_mut(handle.id().into())
                    .unwrap()
                    .as_mut(),
            ),
            data,
        );
        Ok(<C::AssetHandle as AssetHandle>::new(id))
    }

    pub(crate) fn remove<H: AssetHandle>(&mut self, handle: H) -> Result<(), AssetError> {
        let id = handle.id();
        let slot = id.slot();
        if self.entries[slot].version != id.version() {
            return Err(AssetError::AssetNotFound);
        }
        // TODO: remove cached data from container
        if !self.entries[slot].slot.is_null() {
            <H as AssetHandle>::remove_container(
                PrivateAnyAssetContainerMut(self.containers.get_mut(id.into()).unwrap().as_mut()),
                self.entries[slot].slot,
            );
        }
        // Remove entry
        self.remove_entry(slot);
        Ok(())
    }

    pub(crate) fn find<H: AssetHandle>(&self, name: &str) -> Option<H> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.name == name)
            .filter(|(_, entry)| {
                H::check_type(PrivateAnyAssetContainerRef(
                    self.containers
                        .get(entry.component.into())
                        .unwrap()
                        .as_ref(),
                ))
            })
            .map(|(id, entry)| H::new(GenerationId::from_slot(id, entry.version)))
    }

    pub(crate) fn info<H: AssetHandle>(&self, handle: H) -> Result<AssetInfo, AssetError> {
        let id = handle.id();
        self.entries
            .get(id.slot())
            .and_then(|entry| {
                if entry.version == id.version() {
                    Some(AssetInfo { name: &entry.name })
                } else {
                    None
                }
            })
            .ok_or(AssetError::AssetNotFound)
    }

    pub(crate) fn read<H: AssetHandle>(&self, handle: H) -> Result<H::AssetRef<'_>, AssetError> {
        let slot = handle.id().slot();
        let version = handle.id().version();
        let entry = self.entries.get(slot).ok_or(AssetError::AssetNotFound)?;
        if entry.version != version {
            return Err(AssetError::AssetNotFound);
        }
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
        if self.bundles.values().any(|entry| entry.name == name) {
            return Err(AssetError::DuplicatedBundle);
        }
        let version = self.next_version.next();
        let slot = self.bundles.add(AssetBundleEntry {
            name: name.to_owned(),
            first_entry: SlotId::null(),
            version,
        });
        Ok(AssetBundleId::new(GenerationId::from_slot(slot, version)))
    }

    pub(crate) fn remove_bundle(&mut self, bundle: AssetBundleId) -> Result<(), AssetError> {
        let id = bundle.id();
        if let Some(bundle) = self.bundles.get(id.slot()) {
            if bundle.version != id.version() {
                return Err(AssetError::BundleNotFound);
            }
        }
        // Remove all entries
        while !self.bundles[id.slot()].first_entry.is_null() {
            self.remove_entry(self.bundles[id.slot()].first_entry);
        }
        // Remove bundle
        self.bundles.remove(id.slot());
        Ok(())
    }
}
