use std::any::{TypeId, Any};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::marker::PhantomData;

use anyhow::{Result, anyhow, Context};
use serde::de::{Visitor, self, DeserializeSeed};
use serde::ser::{SerializeSeq, SerializeTuple};
use serde::{Serialize, Deserialize, Deserializer, Serializer};

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

struct AssetRegistry<A>(HashMap<UID, AssetEntry<A>>);

impl<A> Default for AssetRegistry<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

trait AnyAssetRegistry: Any {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn merge(&mut self, other: &mut dyn AnyAssetRegistry) -> Result<()>;
    fn serialize_entries<'a>(&'a self, set: &'a HashSet<UID>) -> Box<dyn erased_serde::Serialize + 'a>;
    fn deserialize<'de>(&self, bundle: UID, deserializer: &mut dyn erased_serde::Deserializer<'de>) -> Result<Box<dyn AnyAssetRegistry>>;
}

impl<A: Serialize + for<'de> Deserialize<'de> + 'static> AnyAssetRegistry for AssetRegistry<A> {
    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut (dyn Any + 'static) { self }
    fn merge(&mut self, other: &mut dyn AnyAssetRegistry) -> Result<()> {
        let other: &mut AssetRegistry<A> = other.as_any_mut().downcast_mut().with_context(|| "Invalid asset type cast")?;
        for (uid, entry) in other.0.drain() {
            if self.0.contains_key(&uid) { return Err(anyhow!("Asset '{}' already exists", entry.name)); }
            self.0.insert(uid, entry);
        }
        Ok(())
    }
    fn serialize_entries<'a>(&'a self, set: &'a HashSet<UID>) -> Box<dyn erased_serde::Serialize + 'a> {
        struct AssetRegistrySerialize<'a, A> {
            set: &'a HashSet<UID>,
            registry: &'a AssetRegistry<A>,
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
    fn deserialize<'a>(&self, bundle: UID, deserializer: &mut dyn erased_serde::Deserializer<'a>) -> Result<Box<dyn AnyAssetRegistry>> {
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
        let mut registry: AssetRegistry<A> = Default::default();
        for mut entry in entries.drain(..) {
            let uid = UID::new(&entry.name);
            entry.bundle = bundle;
            registry.0.insert(uid, entry);
        }
        Ok(Box::new(registry))
    }
}

pub struct ImportAssetBundle {
    name: String,
    types: HashMap<TypeId, Box<dyn AnyAssetRegistry>>,
}

struct AssetType {
    name: String,
    registry: Box<dyn AnyAssetRegistry>,
    default: Option<UID>,
}

impl AssetType {
    fn new<A: Serialize + for<'de> Deserialize<'de> + 'static>(name: &str) -> Self {
        Self { name: name.to_string(), registry: Box::new(AssetRegistry::<A>(Default::default())), default: None }
    }
}

struct AssetBundle {
    name: String,
    _owner: UID,
    types: HashMap<TypeId, HashSet<UID>>,
}

impl AssetBundle {
    fn new(name: &str, owner: UID) -> Self {
        Self { name: name.to_string(), _owner: owner, types: Default::default() }
    }
}

#[derive(Default)]
pub struct AssetManager {
    types: HashMap<TypeId, AssetType>,
    uid_to_type: HashMap<UID, TypeId>,
    bundles: HashMap<UID, AssetBundle>,
}

impl AssetManager {

    #[inline]
    fn registry<A: 'static>(&'_ self) -> Result<&'_ AssetRegistry<A>> {
        self.types.get(&TypeId::of::<A>()).map(|t| t.registry.as_any().downcast_ref().unwrap())
            .with_context(|| "Asset type not found")
    }

    #[inline]
    fn registry_mut<A: 'static>(&'_ mut self) -> Result<&'_ mut AssetRegistry<A>> {
        self.types.get_mut(&TypeId::of::<A>()).map(|t| t.registry.as_any_mut().downcast_mut().unwrap())
            .with_context(|| "Asset type not found")
    }

    pub fn set_default<A: 'static>(&mut self, uid: UID) -> Result<()> {
        self.types.get_mut(&TypeId::of::<A>()).with_context(|| "Asset type not found")?
            .default = Some(uid);
        Ok(())
    }

    pub fn get<A: 'static>(&'_ self, uid: UID) -> Result<&'_ A> {
        self.registry::<A>()?.0.get(&uid).map(|entry| &entry.asset)
            .with_context(|| "Asset not found")
    }

    pub fn get_or_default<A: 'static>(&'_ self, uid: UID) -> Result<&'_ A> {
        let asset_type = self.types.get(&TypeId::of::<A>()).with_context(|| "Asset type not found")?;
        let registry: &AssetRegistry<A> = asset_type.registry.as_any().downcast_ref().unwrap();
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
        self.registry_mut::<A>()?.0.get_mut(&uid).map(|entry| &mut entry.asset)
            .with_context(|| "Asset not found")
    }

    pub fn entry<A: 'static>(&'_ self, uid: UID) -> Result<&'_ AssetEntry<A>> {
        self.registry::<A>()?.0.get(&uid)
            .with_context(|| "Asset not found")
    }

    pub fn iter<A: 'static>(&'_ self) -> Result<impl Iterator<Item = (&UID, &'_ AssetEntry<A>)>> {
        Ok(self.registry::<A>()?.0.iter())
    }

    pub fn add_bundle(&mut self, name: &str, owner: UID) -> Result<()> {
        let uid = UID::new(name);
        if self.bundles.contains_key(&uid) { return Err(anyhow!("Bundle already exists")); }
        self.bundles.insert(uid, AssetBundle::new(name, owner));
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

    pub fn deserialize_bundle<'a, D: Deserializer<'a>>(&self, deserializer: D) -> Result<ImportAssetBundle> {
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
                    type Value = HashMap<TypeId, Box<dyn AnyAssetRegistry>>;
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        struct AssetTypesVisitor<'a> {
                            manager: &'a AssetManager,
                        }
                        impl<'de, 'a> Visitor<'de> for AssetTypesVisitor<'a> {
                            type Value = HashMap<TypeId, Box<dyn AnyAssetRegistry>>;
                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                formatter.write_str("Asset types")
                            }
                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                                where A: de::SeqAccess<'de> {
                                struct AssetTypeDeserializeSeed<'a> {
                                    manager: &'a AssetManager,
                                }
                                impl<'de, 'a> DeserializeSeed<'de> for AssetTypeDeserializeSeed<'a> {
                                    type Value = (TypeId, Box<dyn AnyAssetRegistry>);
                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                        where D: Deserializer<'de> {
                                        struct AssetTypeVisitor<'a> {
                                            manager: &'a AssetManager,
                                        }
                                        impl<'de, 'a> Visitor<'de> for AssetTypeVisitor<'a> {
                                            type Value = (TypeId, Box<dyn AnyAssetRegistry>);
                                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                                formatter.write_str("Asset type")
                                            }
                                            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                                                where S: de::SeqAccess<'de> {
                                                struct AssetTypeEntryDeserializeSeed<'a> {
                                                    registry: &'a dyn AnyAssetRegistry,
                                                }
                                                impl<'de, 'a> DeserializeSeed<'de> for AssetTypeEntryDeserializeSeed<'a> {
                                                    type Value = Box<dyn AnyAssetRegistry>;
                                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                                        where D: Deserializer<'de> {
                                                        let mut deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);
                                                        self.registry.deserialize(UID::from(0), &mut deserializer).map_err(Error::custom)
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
        let import = deserializer.deserialize_tuple(2, ImportAssetBundleVisitor { manager: self })
            .map_err(|err| anyhow!("Failed to deserialize bundle: {}", err.to_string()))?;
        Ok(import)
    }

    pub fn import_bundle(&mut self, mut import: ImportAssetBundle) -> Result<()> {
        let uid = UID::new(&import.name);
        if self.bundles.contains_key(&uid) { return Err(anyhow!("Bundle name '{}' already exists", import.name)); }
        for (typeid, assets) in &mut import.types {
            let asset_type = self.types.get_mut(typeid).with_context(|| "Asset type not found")?;
            asset_type.registry.merge(assets.as_mut())?;
        }
        Ok(())
    }

    pub fn register<A: Serialize + for<'a> Deserialize<'a> + 'static>(&mut self, name: &str) -> Result<()> {
        let typeid = TypeId::of::<A>();
        let uid: UID = name.into();
        if self.types.contains_key(&typeid) || self.uid_to_type.contains_key(&uid) {
            return Err(anyhow!("Asset type already registered"));
        }
        self.types.insert(typeid, AssetType::new::<A>(name));
        self.uid_to_type.insert(uid, typeid);
        Ok(())
    }

    pub fn add<A: 'static>(&mut self, name: &str, bundle: UID, data: A) -> Result<()> {
        if !self.bundles.contains_key(&bundle) { return Err(anyhow!("Bundle not found")); }
        let uid = UID::new(name);
        if self.registry::<A>()?.0.contains_key(&uid) { return Err(anyhow!("Asset '{}' already exists", name)); }
        let value = AssetEntry { name: name.to_string(), asset: data, bundle };
        self.registry_mut::<A>()?.0.insert(uid, value);
        let typeid = TypeId::of::<A>();
        self.bundles.get_mut(&bundle).unwrap().types.entry(typeid)
            .or_insert_with(Default::default)
            .insert(uid);
        Ok(())
    }

    pub fn remove<A: 'static>(&mut self, uid: UID) -> Result<()> {
        if !self.registry_mut::<A>()?.0.contains_key(&uid) { return Err(anyhow!("Asset not found")); }
        {
            // Remove from bundle
            let bundle_uid = self.registry_mut::<A>()?.0.get(&uid).unwrap().bundle;
            let bundle = self.bundles.get_mut(&bundle_uid).with_context(|| "Bundle not found")?;
            let typeid = TypeId::of::<A>();
            bundle.types.get_mut(&typeid).with_context(|| "Typeid in bundle was not found")?.remove(&uid);
        }
        {
            // TODO: check dependencies
            self.registry_mut::<A>()?.0.remove(&uid);
        }
        Ok(())
    }

    pub fn transfer<A: 'static>(&mut self, uid: UID, dst_bundle: UID) -> Result<()> {
        let src_bundle = self.registry::<A>()?.0.get(&uid)
            .with_context(|| "Asset not found")?.bundle;
        if !self.bundles.contains_key(&dst_bundle) { return Err(anyhow!("Invalid destination bundle")); }
        if src_bundle == dst_bundle { return Ok(()); }
        let typeid = TypeId::of::<A>();
        self.bundles.get_mut(&src_bundle).with_context(|| "Source bundle not found")?
            .types.get_mut(&typeid).with_context(|| "Typeid in source bundle not found")?
            .remove(&uid);
        self.bundles.get_mut(&dst_bundle)
            .unwrap().types.entry(typeid).or_insert_with(Default::default).insert(uid);
        self.registry_mut::<A>()?.0.get_mut(&uid).unwrap().bundle = dst_bundle;
        Ok(())
    }
}