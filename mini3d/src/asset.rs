use core::result::Result;

use crate::registry::component::{Component, ComponentHandle, ComponentId, ComponentRegistry};
use crate::serialize::{Decoder, DecoderError, Encoder, EncoderError, Serialize};
use crate::utils::slotmap::{DenseSlotMap, SlotId, SparseSecondaryMap};
use crate::utils::uid::UID;

use self::container::{AnyAssetContainer, StaticAssetContainer};
use self::error::AssetError;
use self::handle::{AssetHandle, AssetId, AssetVersion};

pub mod container;
pub mod error;
pub mod handle;

type AssetEntryId = SlotId;
type AssetBundleId = SlotId;

enum AssetSource {
    Persistent,
    IO,
}

pub struct AssetInfo<'a> {
    pub name: &'a str,
}

struct AssetEntry {
    name: UID,
    component: ComponentId,
    version: AssetVersion,
    slot: Option<SlotId>, // None if not loaded
    source: AssetSource,
    bundle: AssetBundleId,
    next_in_bundle: AssetEntryId,
    prev_in_bundle: AssetEntryId,
}

struct AssetBundle {
    name: String,
    first_entry: AssetEntryId,
}

#[derive(Default)]
pub struct AssetManager {
    containers: SparseSecondaryMap<Box<dyn AnyAssetContainer>>, // ComponentId -> Container
    bundles: DenseSlotMap<AssetBundle>,                         // AssetBundleId -> AssetBundle
    entries: DenseSlotMap<AssetEntry>,                          // AssetId -> AssetEntry
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

    pub(crate) fn set_default<H: AssetHandle>(
        &mut self,
        asset: ComponentId,
        uid: UID,
    ) -> Result<(), AssetError> {
        *self
            .defaults
            .get_mut(asset.into())
            .ok_or(AssetError::AssetNotFound)? = uid;
        Ok(())
    }

    pub(crate) fn add<C: ComponentHandle>(
        &mut self,
        name: &str,
        data: <C::AssetHandle as AssetHandle>::Contructor,
    ) -> C::AssetHandle {
    }

    pub(crate) fn find<H: AssetHandle>(&self, name: &str) -> Option<H> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.name == UID::new(&name))
            .filter(|(_, entry)| {
                H::check_type(
                    self.containers
                        .get(entry.component.into())
                        .unwrap()
                        .as_ref(),
                )
            })
            .map(|(id, entry)| H::new(AssetId::new(id, entry.version)))
    }

    pub(crate) fn info<H: AssetHandle>(&self, handle: H) -> Result<AssetInfo, AssetError> {}

    pub(crate) fn read<H: AssetHandle>(
        &mut self,
        handle: H,
    ) -> Result<H::AssetRef<'_>, AssetError> {
        let slot = handle.id().slot();
        let version = handle.id().version();
        let entry = self.entries.get(slot).ok_or(AssetError::AssetNotFound)?;
        if entry.version != version {
            return Err(AssetError::AssetNotFound);
        }
        if let Some(slot) = entry.slot {
            Ok(handle.asset_ref(self.containers[entry.component.into()].as_ref()))
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

    pub(crate) fn add_bundle(&mut self, name: &str) -> Result<UID, AssetError> {
        let uid = UID::new(name);
        if self.bundles.contains_key(&uid) {
            return Err(AssetError::DuplicatedBundleEntry {
                name: name.to_owned(),
            });
        }
        self.bundles.insert(uid, AssetBundle::new(name));
        Ok(uid)
    }

    pub(crate) fn serialize_bundle(
        &self,
        uid: UID,
        registry: &ComponentRegistry,
        encoder: &mut impl Encoder,
    ) -> Result<(), AssetError> {
        let bundle = self.bundles.get(&uid).expect("Bundle not found");
        bundle
            .name
            .serialize(encoder)
            .map_err(|_| AssetError::SerializationError)?;
        encoder
            .write_u32(bundle.assets.len() as u32)
            .map_err(|_| AssetError::SerializationError)?;
        for (asset, set) in bundle.assets.iter() {
            let uid: UID = registry.get(asset.into()).unwrap().name.into();
            uid.serialize(encoder)
                .map_err(|_| AssetError::SerializationError)?;
            let container = self.containers.get(asset).unwrap();
            container.serialize_entries(set, encoder)?;
        }
        Ok(())
    }

    pub(crate) fn add<C: Component>(
        &mut self,
        registry: &ComponentRegistry,
        asset: ComponentId,
        name: &str,
        bundle: UID,
        data: C,
    ) -> Result<(), AssetError> {
        // Check bundle
        if !self.bundles.contains_key(&bundle) {
            return Err(AssetError::BundleNotFound { uid: bundle });
        }
        // Get/Create the container
        if !self.containers.contains(asset.into()) {
            let definition = registry.get(asset).unwrap();
            self.containers
                .insert(asset.into(), definition.reflection.create_asset_container());
        }
        let container = self.containers.get_mut(asset.into()).unwrap();
        // Downcast the container
        let container = container
            .as_any_mut()
            .downcast_mut::<StaticAssetContainer<C>>()
            .ok_or(AssetError::InvalidAssetTypeCast)?;
        // Check if asset already exists
        let uid = UID::new(name);
        if container.0.contains_key(&uid) {
            return Err(AssetError::DuplicatedAssetEntry {
                name: name.to_owned(),
            });
        }
        // Safely insert the asset
        let value = StaticAssetEntry {
            name: name.to_string(),
            asset: data,
            bundle,
        };
        container.0.insert(uid, value);
        self.bundles
            .get_mut(&bundle)
            .unwrap()
            .assets
            .entry(asset.into())
            .or_insert_with(Default::default)
            .insert(uid);
        Ok(())
    }

    pub(crate) fn remove<C: Component>(
        &mut self,
        asset: ComponentId,
        uid: UID,
    ) -> Result<(), AssetError> {
        // Get the container
        let container = self
            .containers
            .get_mut(asset.into())
            .ok_or(AssetError::AssetTypeNotFound)?
            .as_any_mut()
            .downcast_mut::<StaticAssetContainer<C>>()
            .ok_or(AssetError::InvalidAssetTypeCast)?;
        // Remove the asset
        if let Some(entry) = container.0.remove(&uid) {
            self.bundles
                .get_mut(&entry.bundle)
                .expect("Bundle not found")
                .assets
                .get_mut(asset.into())
                .expect("Asset not found")
                .remove(&uid);
        } else {
            return Err(AssetError::AssetNotFound { uid });
        }
        Ok(())
    }

    pub(crate) fn transfer<C: Component>(
        &mut self,
        asset: ComponentId,
        uid: UID,
        dst_bundle: UID,
    ) -> Result<(), AssetError> {
        let src_bundle = self
            .container::<C>(asset)?
            .ok_or(AssetError::AssetTypeNotFound)?
            .0
            .get(&uid)
            .ok_or(AssetError::AssetNotFound { uid })?
            .bundle;
        if !self.bundles.contains_key(&dst_bundle) {
            return Err(AssetError::BundleNotFound { uid: dst_bundle });
        }
        if src_bundle == dst_bundle {
            return Ok(());
        }
        self.bundles
            .get_mut(&src_bundle)
            .ok_or(AssetError::BundleNotFound { uid: src_bundle })?
            .assets
            .get_mut(asset.into())
            .ok_or(AssetError::AssetNotFound { uid })?
            .remove(&uid);
        self.bundles
            .get_mut(&dst_bundle)
            .unwrap()
            .assets
            .entry(asset.into())
            .or_insert_with(Default::default)
            .insert(uid);
        self.container_mut::<C>(asset)?
            .unwrap()
            .0
            .get_mut(&uid)
            .unwrap()
            .bundle = dst_bundle;
        Ok(())
    }
}
