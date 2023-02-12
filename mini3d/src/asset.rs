use std::any::{TypeId, Any};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;

use anyhow::{Result, anyhow, Context};
use serde::de::{Visitor, self, DeserializeSeed};
use serde::ser::{SerializeSeq, SerializeTuple};
use serde::{Serialize, Deserialize, Deserializer, Serializer};

use crate::registry::asset::AssetRegistry;
use crate::uid::UID;

pub struct AssetEntry<A> {
    pub name: String,
    pub asset: A,
    pub bundle: UID,
}

impl<A: Serialize> Serialize for AssetEntry<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.name)?;
        tuple.serialize_element(&self.asset)?;
        tuple.end()
    }
}

impl<'de, A: Deserialize<'de>> Deserialize<'de> for AssetEntry<A> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de> {
        struct AssetEntryVisitor<A> { marker: PhantomData<A> }
        impl<'de, A: Deserialize<'de>> Visitor<'de> for AssetEntryVisitor<A> {
            type Value = AssetEntry<A>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Asset entry")
            }
            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                where S: de::SeqAccess<'de> {
                use serde::de::Error;
                let name: String = seq.next_element()?.with_context(|| "Expect name").map_err(Error::custom)?;
                let asset: A = seq.next_element()?.with_context(|| "Expect asset").map_err(Error::custom)?;
                Ok(AssetEntry { name, asset, bundle: UID::from(0) })
            }
        }
        deserializer.deserialize_tuple(2, AssetEntryVisitor::<A> { marker: PhantomData })
    }
}

struct AssetContainer<A>(HashMap<UID, AssetEntry<A>>);

impl<A> Default for AssetContainer<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub(crate) trait AnyAssetContainer: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn merge(&mut self, other: &mut dyn AnyAssetContainer) -> Result<()>;
    fn collect_uids(&self) -> HashSet<UID>;
    fn clear(&mut self);
    fn serialize_entries<'a>(&'a self, set: &'a HashSet<UID>) -> Box<dyn erased_serde::Serialize + 'a>;
    fn deserialize<'de>(&self, bundle: UID, deserializer: &mut dyn erased_serde::Deserializer<'de>) -> Result<Box<dyn AnyAssetContainer>>;
}

impl<A: Serialize + for<'de> Deserialize<'de> + 'static> AnyAssetContainer for AssetContainer<A> {

    fn as_any(&self) -> &dyn Any { self }

    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) { self }

    fn merge(&mut self, other: &mut dyn AnyAssetContainer) -> Result<()> {
        let other: &mut AssetContainer<A> = other.as_any_mut().downcast_mut().with_context(|| "Invalid asset type cast")?;
        for (uid, entry) in other.0.drain() {
            if self.0.contains_key(&uid) { return Err(anyhow!("Asset '{}' already exists", entry.name)); }
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

    fn serialize_entries<'a>(&'a self, set: &'a HashSet<UID>) -> Box<dyn erased_serde::Serialize + 'a> {
        struct AssetRegistrySerialize<'a, A> {
            set: &'a HashSet<UID>,
            registry: &'a AssetContainer<A>,
        }
        impl<'a, A: Serialize> Serialize for AssetRegistrySerialize<'a, A> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer
            {
                use serde::ser::Error;
                let mut seq = serializer.serialize_seq(Some(self.set.len()))?;
                for uid in self.set {
                    let entry = self.registry.0.get(uid).with_context(|| "Entry not found").map_err(S::Error::custom)?;
                    seq.serialize_element(entry)?;
                }
                seq.end()
            }
        }
        Box::new(AssetRegistrySerialize::<'a, A> { set, registry: self })
    }

    fn deserialize<'a>(&self, bundle: UID, deserializer: &mut dyn erased_serde::Deserializer<'a>) -> Result<Box<dyn AnyAssetContainer>> {
        struct AssetEntryVisitor<A> { marker: PhantomData<A> }
        impl<'de, A: Deserialize<'de>> Visitor<'de> for AssetEntryVisitor<A> {
            type Value = Vec<AssetEntry<A>>;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Sequence of asset entry") 
            }
            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                where S: de::SeqAccess<'de>
            {
                let mut entries: Self::Value = Default::default();
                while let Some(entry) = seq.next_element::<AssetEntry<A>>()? {
                    entries.push(entry);
                }
                Ok(entries)
            }
        }
        // Parse entries
        let mut entries = deserializer.deserialize_seq(AssetEntryVisitor::<A> { marker: PhantomData })?;
        // Build the registry
        let mut container: AssetContainer<A> = Default::default();
        for mut entry in entries.drain(..) {
            let uid = UID::new(&entry.name);
            entry.bundle = bundle;
            container.0.insert(uid, entry);
        }
        Ok(Box::new(container))
    }
}

pub struct ImportAssetBundle {
    name: String,
    types: HashMap<TypeId, Box<dyn AnyAssetContainer>>,
}

struct AssetType {
    name: String,
    registry: Box<dyn AnyAssetContainer>,
    default: Option<UID>,
}

impl AssetType {
    fn new<A: Serialize + for<'de> Deserialize<'de> + 'static>(name: &str) -> Self {
        Self { name: name.to_string(), registry: Box::new(AssetContainer::<A>(Default::default())), default: None }
    }
}

struct AssetBundle {
    name: String,
    types: HashMap<TypeId, HashSet<UID>>,
}

impl AssetBundle {
    fn new(name: &str) -> Self {
        Self { name: name.to_string(), types: Default::default() }
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
    fn container<A: 'static>(&'_ self, registry: &AssetRegistry) -> Result<&'_ AssetContainer<A>> {
        self.containers.get(&registry.uid_from_type::<A>()?)
            .map(|c| c.as_any().downcast_ref().unwrap())
            .with_context(|| "Asset type not found")
    }

    #[inline]
    fn container_mut<A: 'static>(&'_ mut self, registry: &AssetRegistry) -> Result<&'_ mut AssetContainer<A>> {
        self.containers.get_mut(&registry.uid_from_type::<A>()?)
            .map(|c| c.as_any().downcast_mut().unwrap())
            .with_context(|| "Asset type not found")
    }

    pub(crate) fn save_state<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct BundlesSerialize<'a> {
            manager: &'a AssetManager,
        }
        impl<'a> Serialize for BundlesSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                struct BundleSerialize<'a> {
                    manager: &'a AssetManager,
                    uid: UID,
                }
                impl<'a> Serialize for BundleSerialize<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where S: Serializer {
                        self.manager.serialize_bundle(self.uid, serializer)
                    }
                }
                let mut seq = serializer.serialize_seq(Some(self.manager.bundles.len()))?;
                for uid in self.manager.bundles.keys() {
                    seq.serialize_element(&BundleSerialize { uid: *uid, manager: self.manager })?;
                }
                seq.end()
            }
        }
        struct DefaultsSerialize<'a> {
            manager: &'a AssetManager,
        }
        impl<'a> Serialize for DefaultsSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                struct DefaultSerialize {
                    type_name: UID,
                    asset: Option<UID>,
                }
                impl Serialize for DefaultSerialize {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where S: Serializer {
                        let mut tuple = serializer.serialize_tuple(2)?;
                        tuple.serialize_element(&self.type_name)?;
                        tuple.serialize_element(&self.asset)?;
                        tuple.end()
                    }
                }
                let mut seq = serializer.serialize_seq(Some(self.manager.types.len()))?;
                for asset_type in self.manager.types.values() {
                    seq.serialize_element(&DefaultSerialize { type_name: UID::new(&asset_type.name), asset: asset_type.default })?;
                }
                seq.end()
            }
        }
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&BundlesSerialize { manager: self })?;
        tuple.serialize_element(&DefaultsSerialize { manager: self })?;
        tuple.end()
    }

    pub(crate) fn load_state<'de, D: Deserializer<'de>>(&mut self, deserializer: D) -> Result<(), D::Error> {
        struct AssetManagerVisitor<'a> {
            manager: &'a mut AssetManager,
        }
        impl<'de, 'a> Visitor<'de> for AssetManagerVisitor<'a> {
            type Value = ();
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Asset manager")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: de::SeqAccess<'de> {
                struct BundlesDeserializeSeed<'a> {
                    manager: &'a mut AssetManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for BundlesDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        struct BundlesVisitor<'a> {
                            manager: &'a mut AssetManager,
                        }
                        impl<'de, 'a> Visitor<'de> for BundlesVisitor<'a> {
                            type Value = ();
                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                formatter.write_str("Bundles")
                            }
                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                                where A: de::SeqAccess<'de> {
                                use serde::de::Error;
                                struct BundleDeserializeSeed<'a> {
                                    manager: &'a mut AssetManager,
                                }
                                impl<'de, 'a> DeserializeSeed<'de> for BundleDeserializeSeed<'a> {
                                    type Value = ImportAssetBundle;
                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                        where D: Deserializer<'de> {
                                        self.manager.deserialize_bundle(deserializer)
                                    }
                                }
                                while let Some(import) = seq.next_element_seed(BundleDeserializeSeed { manager: self.manager })? {
                                    self.manager.import_bundle(import).map_err(Error::custom)?;
                                }
                                Ok(())
                            }
                        }
                        self.manager.bundles.clear();
                        for asset_type in self.manager.types.values_mut() {
                            asset_type.registry.clear();
                            asset_type.default = None;
                        }
                        deserializer.deserialize_seq(BundlesVisitor { manager: self.manager })
                    }
                }
                struct DefaultsDeserializeSeed<'a> {
                    manager: &'a mut AssetManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for DefaultsDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        struct DefaultsVisitor<'a> {
                            manager: &'a mut AssetManager,
                        }
                        impl<'de, 'a> Visitor<'de> for DefaultsVisitor<'a> {
                            type Value = ();
                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                formatter.write_str("Defaults")
                            }
                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                                where A: de::SeqAccess<'de> {
                                struct DefaultDeserializeSeed<'a> {
                                    manager: &'a mut AssetManager,
                                }
                                impl<'de, 'a> DeserializeSeed<'de> for DefaultDeserializeSeed<'a> {
                                    type Value = ();
                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                        where D: Deserializer<'de> {
                                        struct DefaultVisitor<'a> {
                                            manager: &'a mut AssetManager,
                                        }
                                        impl<'de, 'a> Visitor<'de> for DefaultVisitor<'a> {
                                            type Value = ();
                                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                                formatter.write_str("Default entry")
                                            }
                                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                                                where A: de::SeqAccess<'de> {
                                                let name: UID = seq.next_element()?.with_context(|| "Expect typename").map_err(Error::custom)?;
                                                let asset: Option<UID> = seq.next_element()?.with_context(|| "Expect default asset (option)").map_err(Error::custom)?;
                                                let type_id = self.manager.uid_to_type.get(&name).with_context(|| "Type id not found").map_err(Error::custom)?;
                                                let asset_type = self.manager.types.get_mut(type_id).with_context(|| "Asset type not found").map_err(Error::custom)?;
                                                asset_type.default = asset;
                                                Ok(())
                                            }
                                        }
                                        deserializer.deserialize_tuple(2, DefaultVisitor { manager: self.manager })
                                    }
                                }
                                use serde::de::Error;
                                while seq.next_element_seed(DefaultDeserializeSeed { manager: self.manager })?.is_some() {}
                                Ok(())
                            }
                        }
                        deserializer.deserialize_seq(DefaultsVisitor { manager: self.manager })
                    }
                }
                seq.next_element_seed(BundlesDeserializeSeed { manager: self.manager })?;
                seq.next_element_seed(DefaultsDeserializeSeed { manager: self.manager })?;
                Ok(())
            }
        }
        deserializer.deserialize_tuple(2, AssetManagerVisitor { manager: self })?;
        Ok(())
    }

    pub fn set_default<A: 'static>(&mut self, uid: UID) -> Result<()> {
        self.types.get_mut(&TypeId::of::<A>()).with_context(|| "Asset type not found")?
            .default = Some(uid);
        Ok(())
    }

    pub fn get<A: 'static>(&'_ self, uid: UID) -> Result<&'_ A> {
        self.container::<A>()?.0.get(&uid).map(|entry| &entry.asset)
            .with_context(|| "Asset not found")
    }

    pub fn get_or_default<A: 'static>(&'_ self, uid: UID) -> Result<&'_ A> {
        let asset_type = self.types.get(&TypeId::of::<A>()).with_context(|| "Asset type not found")?;
        let registry: &AssetContainer<A> = asset_type.registry.as_any().downcast_ref().unwrap();
        registry.0.get(&uid)
        .or_else(|| {
            asset_type.default.and_then(|uid| {
                registry.0.get(&uid)
            })
        })
        .map(|entry| &entry.asset)
        .with_context(|| "Asset not found and no default provided")
    }

    pub fn get_mut<A: 'static>(&'_ mut self, uid: UID) -> Result<&'_ mut A> {
        self.container_mut::<A>()?.0.get_mut(&uid).map(|entry| &mut entry.asset)
            .with_context(|| "Asset not found")
    }

    pub fn entry<A: 'static>(&'_ self, uid: UID) -> Result<&'_ AssetEntry<A>> {
        self.container::<A>()?.0.get(&uid)
            .with_context(|| "Asset not found")
    }

    pub fn iter<A: 'static>(&'_ self) -> Result<impl Iterator<Item = (&UID, &'_ AssetEntry<A>)>> {
        Ok(self.container::<A>()?.0.iter())
    }

    pub fn add_bundle(&mut self, name: &str) -> Result<()> {
        let uid = UID::new(name);
        if self.bundles.contains_key(&uid) { return Err(anyhow!("Bundle already exists")); }
        self.bundles.insert(uid, AssetBundle::new(name));
        Ok(())
    }

    pub fn serialize_bundle<S: Serializer>(&self, uid: UID, serializer: S) -> Result<S::Ok, S::Error> {
        struct AssetTypesSerialize<'a> {
            manager: &'a AssetManager,
            types: &'a HashMap<TypeId, HashSet<UID>>,
        }
        impl<'a> Serialize for AssetTypesSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                struct AssetTypeSerialize<'a> {
                    asset_type: &'a AssetType,
                    set: &'a HashSet<UID>,
                }
                impl<'a> Serialize for AssetTypeSerialize<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where S: Serializer {
                        let mut tuple = serializer.serialize_tuple(2)?;
                        tuple.serialize_element(&self.asset_type.name)?;
                        tuple.serialize_element(&self.asset_type.registry.serialize_entries(self.set))?;
                        tuple.end()
                    }
                }
                let mut seq = serializer.serialize_seq(Some(self.types.len()))?;
                for (type_id, set) in self.types.iter() {
                    let asset_type = self.manager.types.get(type_id).with_context(|| "Asset type not found").map_err(Error::custom)?;
                    seq.serialize_element(&AssetTypeSerialize { asset_type, set })?;
                }
                seq.end()
            }
        }
        use serde::ser::Error;
        let bundle = self.bundles.get(&uid).with_context(|| "Bundle not found").map_err(S::Error::custom)?;
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&bundle.name)?;
        tuple.serialize_element(&AssetTypesSerialize { types: &bundle.types, manager: self })?;
        tuple.end()
    }

    pub fn deserialize_bundle<'a, D: Deserializer<'a>>(&self, deserializer: D) -> Result<ImportAssetBundle, D::Error> {
        struct ImportAssetBundleVisitor<'a> {
            manager: &'a AssetManager,
        }
        impl<'de, 'a> Visitor<'de> for ImportAssetBundleVisitor<'a> {
            type Value = ImportAssetBundle;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Import asset bundle")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: de::SeqAccess<'de> 
            {
                struct AssetTypesDeserializeSeed<'a> {
                    manager: &'a AssetManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for AssetTypesDeserializeSeed<'a> {
                    type Value = HashMap<TypeId, Box<dyn AnyAssetContainer>>;
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        struct AssetTypesVisitor<'a> {
                            manager: &'a AssetManager,
                        }
                        impl<'de, 'a> Visitor<'de> for AssetTypesVisitor<'a> {
                            type Value = HashMap<TypeId, Box<dyn AnyAssetContainer>>;
                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                formatter.write_str("Asset types")
                            }
                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                                where A: de::SeqAccess<'de> {
                                struct AssetTypeDeserializeSeed<'a> {
                                    manager: &'a AssetManager,
                                }
                                impl<'de, 'a> DeserializeSeed<'de> for AssetTypeDeserializeSeed<'a> {
                                    type Value = (TypeId, Box<dyn AnyAssetContainer>);
                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                        where D: Deserializer<'de> {
                                        struct AssetTypeVisitor<'a> {
                                            manager: &'a AssetManager,
                                        }
                                        impl<'de, 'a> Visitor<'de> for AssetTypeVisitor<'a> {
                                            type Value = (TypeId, Box<dyn AnyAssetContainer>);
                                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                                formatter.write_str("Asset type")
                                            }
                                            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                                                where S: de::SeqAccess<'de> {
                                                struct AssetTypeEntryDeserializeSeed<'a> {
                                                    registry: &'a dyn AnyAssetContainer,
                                                }
                                                impl<'de, 'a> DeserializeSeed<'de> for AssetTypeEntryDeserializeSeed<'a> {
                                                    type Value = Box<dyn AnyAssetContainer>;
                                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                                        where D: Deserializer<'de> {
                                                        let mut deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);
                                                        self.registry.deserialize(UID::null(), &mut deserializer).map_err(Error::custom)
                                                    }
                                                }
                                                let name: String = seq.next_element()?.with_context(|| "Expect asset type name").map_err(Error::custom)?;
                                                let uid = UID::new(&name);
                                                let type_id = self.manager.uid_to_type.get(&uid)
                                                    .with_context(|| format!("Asset type '{}' not found", name)).map_err(Error::custom)?;
                                                let asset_type = self.manager.types.get(type_id)
                                                    .with_context(|| "Asset type not found from UID").map_err(Error::custom)?;
                                                let registry = seq.next_element_seed(AssetTypeEntryDeserializeSeed { registry: asset_type.registry.as_ref() })
                                                    ?.with_context(|| "Expect asset entries").map_err(Error::custom)?;
                                                Ok((*type_id, registry))
                                            }
                                        }
                                        deserializer.deserialize_tuple(2, AssetTypeVisitor { manager: self.manager })
                                    }
                                }
                                let mut hashmap: Self::Value = Default::default();
                                while let Some((type_id, registry)) = seq.next_element_seed(AssetTypeDeserializeSeed { manager: self.manager })? {
                                    if hashmap.contains_key(&type_id) {
                                        return Err(Error::custom("Asset type already exists"));
                                    }
                                    hashmap.insert(type_id, registry);
                                }
                                Ok(hashmap)
                            }
                        }
                        deserializer.deserialize_seq(AssetTypesVisitor { manager: self.manager })
                    }
                }
                use serde::de::Error;
                let name: String = seq.next_element()?.with_context(|| "Expect name").map_err(Error::custom)?;
                let types = seq.next_element_seed(AssetTypesDeserializeSeed { manager: self.manager })?
                    .with_context(|| "Expect types").map_err(Error::custom)?;
                Ok(ImportAssetBundle { name, types })
            }
        }
        deserializer.deserialize_tuple(2, ImportAssetBundleVisitor { manager: self })
    }

    pub fn import_bundle(&mut self, mut import: ImportAssetBundle) -> Result<()> {
        self.add_bundle(&import.name)?;
        let bundle = self.bundles.get_mut(&UID::new(&import.name)).expect("Bundle not found");
        for (type_id, assets) in &mut import.types {
            let asset_type = self.types.get_mut(type_id).with_context(|| "Asset type not found")?;
            bundle.types.insert(*type_id, assets.collect_uids());
            asset_type.registry.merge(assets.as_mut())?;
        }
        Ok(())
    }

    pub fn add<A: 'static>(&mut self, name: &str, bundle: UID, data: A) -> Result<()> {
        if !self.bundles.contains_key(&bundle) { return Err(anyhow!("Bundle not found")); }
        let uid = UID::new(name);
        if self.container::<A>()?.0.contains_key(&uid) { return Err(anyhow!("Asset '{}' already exists", name)); }
        let value = AssetEntry { name: name.to_string(), asset: data, bundle };
        self.container_mut::<A>()?.0.insert(uid, value);
        let type_id = TypeId::of::<A>();
        self.bundles.get_mut(&bundle).unwrap().types.entry(type_id)
            .or_insert_with(Default::default)
            .insert(uid);
        Ok(())
    }

    pub fn remove<A: 'static>(&mut self, uid: UID) -> Result<()> {
        if !self.container_mut::<A>()?.0.contains_key(&uid) { return Err(anyhow!("Asset not found")); }
        {
            // Remove from bundle
            let bundle_uid = self.container_mut::<A>()?.0.get(&uid).unwrap().bundle;
            let bundle = self.bundles.get_mut(&bundle_uid).with_context(|| "Bundle not found")?;
            let type_id = TypeId::of::<A>();
            bundle.types.get_mut(&type_id).with_context(|| "Typeid in bundle was not found")?.remove(&uid);
        }
        {
            // TODO: check dependencies
            self.container_mut::<A>()?.0.remove(&uid);
        }
        Ok(())
    }

    pub fn transfer<A: 'static>(&mut self, uid: UID, dst_bundle: UID) -> Result<()> {
        let src_bundle = self.container::<A>()?.0.get(&uid)
            .with_context(|| "Asset not found")?.bundle;
        if !self.bundles.contains_key(&dst_bundle) { return Err(anyhow!("Invalid destination bundle")); }
        if src_bundle == dst_bundle { return Ok(()); }
        let type_id = TypeId::of::<A>();
        self.bundles.get_mut(&src_bundle).with_context(|| "Source bundle not found")?
            .types.get_mut(&type_id).with_context(|| "Typeid in source bundle not found")?
            .remove(&uid);
        self.bundles.get_mut(&dst_bundle)
            .unwrap().types.entry(type_id).or_insert_with(Default::default).insert(uid);
        self.container_mut::<A>()?.0.get_mut(&uid).unwrap().bundle = dst_bundle;
        Ok(())
    }
}