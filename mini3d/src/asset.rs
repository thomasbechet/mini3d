use std::any::Any;
use std::collections::{HashMap, HashSet, hash_map};
use core::result::Result;

use mini3d_derive::Error;

use crate::registry::asset::{Asset, AssetRegistry};
use crate::serialize::{Serialize, Encoder, EncoderError, Decoder, DecoderError};
use crate::uid::UID;

#[derive(Debug, Error)]
pub enum AssetError {
    #[error("Duplicated asset entry: {name}")]
    DuplicatedAssetEntry { name: String },
    #[error("Duplicated asset type: {uid}")]
    DuplicatedAssetType { uid: UID },
    #[error("Invalid asset type cast")]
    InvalidAssetTypeCast,
    #[error("Asset not found: {uid}")]
    AssetNotFound { uid: UID },
    #[error("Asset type not found: {uid}")]
    AssetTypeNotFound { uid: UID },
    #[error("Bundle not found: {uid}")]
    BundleNotFound { uid: UID },
    #[error("Duplicated bundle entry: {name}")]
    DuplicatedBundleEntry { name: String },
    #[error("Deserialization error")]
    DeserializationError,
    #[error("Serialization error")]
    SerializationError,
}

pub struct AssetEntry<A> {
    pub name: String,
    pub asset: A,
    pub bundle: UID,
}

impl<A: Asset> AssetEntry<A> {
    pub fn uid(&self) -> UID {
        UID::new(&self.name)
    }
}

impl<A: Asset> Serialize for AssetEntry<A> {

    type Header = A::Header;

    fn serialize(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        self.name.serialize(encoder)?;
        self.asset.serialize(encoder)?;
        Ok(())
    }

    fn deserialize(decoder: &mut impl Decoder, header: &Self::Header) -> Result<Self, DecoderError> {
        let name = String::deserialize(decoder, &Default::default())?;
        let asset = A::deserialize(decoder, header)?;
        Ok(AssetEntry { name, asset, bundle: UID::default() })
    }
}

pub(crate) struct AssetContainer<A: Asset>(HashMap<UID, AssetEntry<A>>);

impl<A: Asset> Default for AssetContainer<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub(crate) trait AnyAssetContainer: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn merge(&mut self, other: &mut dyn AnyAssetContainer) -> Result<(), AssetError>;
    fn collect_uids(&self) -> HashSet<UID>;
    fn clear(&mut self);
    fn serialize_entries(&self, set: &HashSet<UID>, encoder: &mut dyn Encoder) -> Result<(), AssetError>;
    fn deserialize_entries(&mut self, bundle: UID, decoder: &mut dyn Decoder) -> Result<(), AssetError>;
}

impl<A: Asset> AnyAssetContainer for AssetContainer<A> {

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) { self }

    fn merge(&mut self, other: &mut dyn AnyAssetContainer) -> Result<(), AssetError> {
        let other: &mut AssetContainer<A> = other.as_any_mut().downcast_mut().ok_or(AssetError::InvalidAssetTypeCast)?;
        for (uid, entry) in other.0.drain() {
            if self.0.contains_key(&uid) { return Err(AssetError::DuplicatedAssetEntry { name: entry.name }); }
            self.0.insert(uid, entry);
        }
        Ok(())
    }

    fn collect_uids(&self) -> HashSet<UID> {
        self.0.keys().copied().collect::<HashSet<UID>>()
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    fn serialize_entries(&self, set: &HashSet<UID>, mut encoder: &mut dyn Encoder) -> Result<(), AssetError> {
        A::Header::default().serialize(&mut encoder).map_err(|_| AssetError::SerializationError)?;
        encoder.write_u32(set.len() as u32).map_err(|_| AssetError::SerializationError)?;
        for uid in set {
            let entry = self.0.get(uid).ok_or(AssetError::AssetNotFound { uid: *uid })?;
            entry.serialize(&mut encoder).map_err(|_| AssetError::SerializationError)?;
        }
        Ok(())
    }

    fn deserialize_entries(&mut self, bundle: UID, mut decoder: &mut dyn Decoder) -> Result<(), AssetError> {
        let header = A::Header::deserialize(&mut decoder, &Default::default()).map_err(|_| AssetError::DeserializationError)?;
        let len = decoder.read_u32().map_err(|_| AssetError::DeserializationError)? as usize;
        for _ in 0..len {
            let mut entry = AssetEntry::<A>::deserialize(&mut decoder, &header).map_err(|_| AssetError::DeserializationError)?;
            entry.bundle = bundle;
            self.0.insert(entry.uid(), entry);
        }
        Ok(())
    }
}

pub struct ImportAssetBundle {
    name: String,
    containers: HashMap<UID, Box<dyn AnyAssetContainer>>,
}

impl ImportAssetBundle {

    pub(crate) fn deserialize(registry: &AssetRegistry, decoder: &mut impl Decoder) -> Result<ImportAssetBundle, AssetError> {
        let name = String::deserialize(decoder, &Default::default()).map_err(|_| AssetError::DeserializationError)?;
        let bundle = UID::new(&name);
        let len = decoder.read_u32().map_err(|_| AssetError::DeserializationError)? as usize;
        let mut containers: HashMap<UID, Box<dyn AnyAssetContainer>> = Default::default();
        for _ in 0..len {
            let asset = UID::deserialize(decoder, &()).map_err(|_| AssetError::DeserializationError)?;
            let definition = registry.get(asset).map_err(|_| AssetError::AssetTypeNotFound { uid: asset })?;
            let mut container = definition.reflection.create_container();
            container.deserialize_entries(bundle, decoder)?;
            if containers.contains_key(&asset) {
                return Err(AssetError::DuplicatedAssetType { uid: asset });
            }
            containers.insert(asset, container);
        }
        Ok(ImportAssetBundle { name, containers })
    }
}

struct AssetBundle {
    name: String,
    assets: HashMap<UID, HashSet<UID>>,
}

impl AssetBundle {
    fn new(name: &str) -> Self {
        Self { name: name.to_string(), assets: Default::default() }
    }
}

#[derive(Default)]
pub struct AssetManager {
    containers: HashMap<UID, Box<dyn AnyAssetContainer>>,
    defaults: HashMap<UID, UID>,
    bundles: HashMap<UID, AssetBundle>,
}

impl AssetManager {

    #[inline]
    fn container<A: Asset>(&'_ self, asset: UID) -> Result<Option<&'_ AssetContainer<A>>, AssetError> {
        if let Some(container) = self.containers.get(&asset) {
            return Ok(Some(container.as_any().downcast_ref().ok_or(AssetError::InvalidAssetTypeCast)?));
        }
        Ok(None)
    }

    #[inline]
    fn container_mut<A: Asset>(&'_ mut self, asset: UID) -> Result<Option<&'_ mut AssetContainer<A>>, AssetError> {
        if let Some(container) = self.containers.get_mut(&asset) {
            return Ok(Some(container.as_any_mut().downcast_mut().ok_or(AssetError::InvalidAssetTypeCast)?));
        }
        Ok(None)
    }

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        encoder.write_u32(self.bundles.len() as u32)?;
        for uid in self.bundles.keys() {
            self.serialize_bundle(*uid, encoder).map_err(|_| EncoderError::Unsupported)?;
        }
        self.defaults.serialize(encoder)?;
        Ok(())
    }

    pub(crate) fn load_state(&mut self, registry: &AssetRegistry, decoder: &mut impl Decoder) -> Result<(), DecoderError> {

        self.bundles.clear();
        self.containers.clear();
        self.defaults.clear();
        
        let bundle_count = decoder.read_u32()?;
        for _ in 0..bundle_count {
            let import = ImportAssetBundle::deserialize(registry, decoder).map_err(|_| DecoderError::CorruptedData)?;
            self.import_bundle(import).map_err(|_| DecoderError::CorruptedData)?;
        }
        
        self.defaults = HashMap::deserialize(decoder, &Default::default())?;
        for asset in self.defaults.keys() {
            if self.containers.get(asset).is_none() {
                return Err(DecoderError::CorruptedData);
            }
        }
        Ok(())
    }

    pub(crate) fn set_default(&mut self, asset: UID, uid: UID) -> Result<(), AssetError> {
        *self.defaults.get_mut(&asset).ok_or(AssetError::AssetNotFound { uid })? = uid;
        Ok(())
    }

    pub(crate) fn get<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<Option<&'_ A>, AssetError> {
        Ok(self.container::<A>(asset)?.and_then(|container| container.0.get(&uid).map(|entry| &entry.asset)))
    }

    pub(crate) fn get_or_default<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<Option<&'_ A>, AssetError> {
        let container = self.container::<A>(asset)?;
        Ok(container.and_then(|container| {
            container.0.get(&uid).or_else(|| {
                self.defaults.get(&asset).and_then(|uid| container.0.get(uid))
            }).map(|entry| &entry.asset)
        }))
    }

    pub(crate) fn entry<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<Option<&'_ AssetEntry<A>>, AssetError> {
        Ok(self.container::<A>(asset)?.and_then(|container| container.0.get(&uid)))
    }

    pub(crate) fn iter<A: Asset>(&self, asset: UID) -> Result<Option<impl Iterator<Item = &AssetEntry<A>>>, AssetError> {
        Ok(self.container::<A>(asset)?.map(|container| container.0.values()))
    }

    pub(crate) fn add_bundle(&mut self, name: &str) -> Result<UID, AssetError> {
        let uid = UID::new(name);
        if self.bundles.contains_key(&uid) { return Err(AssetError::DuplicatedBundleEntry { name: name.to_owned() }); }
        self.bundles.insert(uid, AssetBundle::new(name));
        Ok(uid)
    }

    pub(crate) fn serialize_bundle(&self, uid: UID, encoder: &mut impl Encoder) -> Result<(), AssetError> {
        let bundle = self.bundles.get(&uid).expect("Bundle not found");
        bundle.name.serialize(encoder).map_err(|_| AssetError::SerializationError)?;
        encoder.write_u32(bundle.assets.len() as u32).map_err(|_| AssetError::SerializationError)?;
        for (asset, set) in &bundle.assets {
            asset.serialize(encoder).map_err(|_| AssetError::SerializationError)?;
            let container = self.containers.get(asset).unwrap();
            container.serialize_entries(set, encoder)?;
        }
        Ok(())
    }

    pub(crate) fn import_bundle(&mut self, import: ImportAssetBundle) -> Result<(), AssetError> {
        let uid = self.add_bundle(&import.name)?;
        let bundle = self.bundles.get_mut(&uid).unwrap();
        for (asset, mut container) in import.containers {
            bundle.assets.insert(asset, container.collect_uids());
            if let Some(self_container) = self.containers.get_mut(&asset) {
                self_container.merge(container.as_mut())?;
            } else {
                self.containers.insert(asset, container);
            }
        }
        Ok(())
    }

    pub(crate) fn add<A: Asset>(&mut self, registry: &AssetRegistry, asset: UID, name: &str, bundle: UID, data: A) -> Result<(), AssetError> {
        // Check bundle
        if !self.bundles.contains_key(&bundle) { return Err(AssetError::BundleNotFound { uid: bundle }); }
        // Get/Create the container
        let container = match self.containers.entry(asset) {
            hash_map::Entry::Occupied(entry) => {
                entry.into_mut()
            },
            hash_map::Entry::Vacant(entry) => {
                let definition = registry.get(asset).unwrap();
                entry.insert(definition.reflection.create_container())
            },
        };
        // Downcast the container
        let container = container.as_any_mut().downcast_mut::<AssetContainer<A>>().ok_or(AssetError::InvalidAssetTypeCast)?;
        // Check if asset already exists
        let uid = UID::new(name);
        if container.0.contains_key(&uid) { return Err(AssetError::DuplicatedAssetEntry { name: name.to_owned() }); }
        // Safely insert the asset
        let value = AssetEntry { name: name.to_string(), asset: data, bundle };
        container.0.insert(uid, value);
        self.bundles.get_mut(&bundle).unwrap().assets.entry(asset)
            .or_insert_with(Default::default)
            .insert(uid);
        Ok(())
    }

    pub(crate) fn remove<A: Asset>(&mut self, asset: UID, uid: UID) -> Result<(), AssetError> {
        // Get the container
        let container = self.containers.get_mut(&asset).ok_or(AssetError::AssetTypeNotFound { uid: asset })?
            .as_any_mut().downcast_mut::<AssetContainer<A>>().ok_or(AssetError::InvalidAssetTypeCast)?;
        // Remove the asset
        if let Some(entry) = container.0.remove(&uid) {
            self.bundles.get_mut(&entry.bundle).expect("Bundle not found")
                .assets.get_mut(&asset).expect("Asset not found")
                .remove(&uid);
        } else {
            return Err(AssetError::AssetNotFound { uid });
        }
        Ok(())
    }

    pub(crate) fn transfer<A: Asset>(&mut self, asset: UID, uid: UID, dst_bundle: UID) -> Result<(), AssetError> {
        let src_bundle = self.container::<A>(asset)?.ok_or(AssetError::AssetTypeNotFound { uid: asset })?
            .0.get(&uid).ok_or(AssetError::AssetNotFound { uid })?.bundle;
        if !self.bundles.contains_key(&dst_bundle) { return Err(AssetError::BundleNotFound { uid: dst_bundle }); }
        if src_bundle == dst_bundle { return Ok(()); }
        self.bundles.get_mut(&src_bundle).ok_or(AssetError::BundleNotFound { uid: src_bundle })?
            .assets.get_mut(&asset).ok_or(AssetError::AssetNotFound { uid })?
            .remove(&uid);
        self.bundles.get_mut(&dst_bundle)
            .unwrap().assets.entry(asset).or_insert_with(Default::default).insert(uid);
        self.container_mut::<A>(asset)?.unwrap().0.get_mut(&uid).unwrap().bundle = dst_bundle;
        Ok(())
    }
}