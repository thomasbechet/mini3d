use std::any::Any;
use std::collections::{HashMap, HashSet, hash_map};
use std::fmt;
use std::marker::PhantomData;

use anyhow::{Result, anyhow, Context};
use serde::de::{Visitor, self, DeserializeSeed};
use serde::ser::{SerializeSeq, SerializeTuple};
use serde::{Serialize, Deserialize, Deserializer, Serializer};

use crate::registry::asset::{Asset, AssetRegistry};
use crate::uid::UID;

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
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer {
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&self.name)?;
        tuple.serialize_element(&self.asset)?;
        tuple.end()
    }
}

impl<'de, A: Asset> Deserialize<'de> for AssetEntry<A> {
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

pub(crate) struct AssetContainer<A: Asset>(HashMap<UID, AssetEntry<A>>);

impl<A: Asset> Default for AssetContainer<A> {
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
    fn deserialize_entries<'de>(&mut self, bundle: UID, deserializer: &mut dyn erased_serde::Deserializer<'de>) -> Result<()>;
}

impl<A: Asset> AnyAssetContainer for AssetContainer<A> {

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
        struct AssetRegistrySerialize<'a, A: Asset> {
            set: &'a HashSet<UID>,
            registry: &'a AssetContainer<A>,
        }
        impl<'a, A: Asset> Serialize for AssetRegistrySerialize<'a, A> {
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

    fn deserialize_entries<'a>(&mut self, bundle: UID, deserializer: &mut dyn erased_serde::Deserializer<'a>) -> Result<()> {
        struct AssetEntryVisitor<A> { marker: PhantomData<A> }
        impl<'de, A: Asset> Visitor<'de> for AssetEntryVisitor<A> {
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
        for mut entry in entries.drain(..) {
            let asset = UID::new(&entry.name);
            entry.bundle = bundle;
            self.0.insert(asset, entry);
        }
        Ok(())
    }
}

pub struct ImportAssetBundle {
    name: String,
    containers: HashMap<UID, Box<dyn AnyAssetContainer>>,
}

impl ImportAssetBundle {

    pub(crate) fn deserialize<'a, D: Deserializer<'a>>(registry: &AssetRegistry, deserializer: D) -> Result<ImportAssetBundle, D::Error> {
        struct ImportAssetBundleVisitor<'a> {
            registry: &'a AssetRegistry,
        }
        impl<'de, 'a> Visitor<'de> for ImportAssetBundleVisitor<'a> {
            type Value = ImportAssetBundle;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Import asset bundle")
            }
            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                where A: de::SeqAccess<'de> 
            {
                struct ContainersDeserializeSeed<'a> {
                    registry: &'a AssetRegistry,
                    bundle: UID,
                }
                impl<'de, 'a> DeserializeSeed<'de> for ContainersDeserializeSeed<'a> {
                    type Value = HashMap<UID, Box<dyn AnyAssetContainer>>;
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        struct ContainersVisitor<'a> {
                            registry: &'a AssetRegistry,
                            bundle: UID,
                        }
                        impl<'de, 'a> Visitor<'de> for ContainersVisitor<'a> {
                            type Value = HashMap<UID, Box<dyn AnyAssetContainer>>;
                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                formatter.write_str("Containers")
                            }
                            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
                                where A: de::SeqAccess<'de> {
                                struct AssetContainerDeserializeSeed<'a> {
                                    registry: &'a AssetRegistry,
                                    bundle: UID,
                                }
                                impl<'de, 'a> DeserializeSeed<'de> for AssetContainerDeserializeSeed<'a> {
                                    type Value = (UID, Box<dyn AnyAssetContainer>);
                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                        where D: Deserializer<'de> {
                                        struct AssetContainerVisitor<'a> {
                                            registry: &'a AssetRegistry,
                                            bundle: UID,
                                        }
                                        impl<'de, 'a> Visitor<'de> for AssetContainerVisitor<'a> {
                                            type Value = (UID, Box<dyn AnyAssetContainer>);
                                            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                                                formatter.write_str("Container")
                                            }
                                            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
                                                where S: de::SeqAccess<'de> {
                                                struct ContainerDeserializeSeed<'a> {
                                                    container: &'a mut dyn AnyAssetContainer,
                                                    bundle: UID,
                                                }
                                                impl<'de, 'a> DeserializeSeed<'de> for ContainerDeserializeSeed<'a> {
                                                    type Value = ();
                                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                                        where D: Deserializer<'de> {
                                                        let mut deserializer = <dyn erased_serde::Deserializer>::erase(deserializer);
                                                        self.container.deserialize_entries(self.bundle, &mut deserializer).map_err(Error::custom)
                                                    }
                                                }
                                                let asset: UID = seq.next_element()?.with_context(|| "Expect asset type UID").map_err(Error::custom)?;
                                                let definition = self.registry.get(asset)
                                                    .with_context(|| "Asset type not defined").map_err(Error::custom)?;
                                                let mut container = definition.reflection.create_container();
                                                seq.next_element_seed(ContainerDeserializeSeed { container: container.as_mut(), bundle: self.bundle })?
                                                    .with_context(|| "Expect asset entries").map_err(Error::custom)?;
                                                Ok((asset, container))
                                            }
                                        }
                                        deserializer.deserialize_tuple(2, AssetContainerVisitor { registry: self.registry, bundle: self.bundle })
                                    }
                                }
                                let mut hashmap: Self::Value = Default::default();
                                while let Some((asset, container)) = seq.next_element_seed(AssetContainerDeserializeSeed { registry: self.registry, bundle: self.bundle })? {
                                    if hashmap.contains_key(&asset) {
                                        return Err(Error::custom("Container already exists"));
                                    }
                                    hashmap.insert(asset, container);
                                }
                                Ok(hashmap)
                            }
                        }
                        deserializer.deserialize_seq(ContainersVisitor { registry: self.registry, bundle: self.bundle })
                    }
                }
                use serde::de::Error;
                let name: String = seq.next_element()?.with_context(|| "Expect name").map_err(Error::custom)?;
                let bundle: UID = name.as_str().into();
                let containers = seq.next_element_seed(ContainersDeserializeSeed { registry: self.registry, bundle })?
                    .with_context(|| "Expect types").map_err(Error::custom)?;
                Ok(ImportAssetBundle { name, containers })
            }
        }
        deserializer.deserialize_tuple(2, ImportAssetBundleVisitor { registry })
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
    fn container<A: Asset>(&'_ self, asset: UID) -> Result<&'_ AssetContainer<A>> {
        self.containers.get(&asset).with_context(|| "Asset type not found")?
            .as_any().downcast_ref().with_context(|| "Asset type mismatch")
    }

    #[inline]
    fn container_mut<A: Asset>(&'_ mut self, asset: UID) -> Result<&'_ mut AssetContainer<A>> {
        self.containers.get_mut(&asset).with_context(|| "Asset type not found")?
            .as_any_mut().downcast_mut().with_context(|| "Asset type mismatch")
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
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&BundlesSerialize { manager: self })?;
        tuple.serialize_element(&self.defaults)?;
        tuple.end()
    }

    pub(crate) fn load_state<'de, D: Deserializer<'de>>(&mut self, registry: &AssetRegistry, deserializer: D) -> Result<(), D::Error> {
        struct AssetManagerVisitor<'a> {
            registry: &'a AssetRegistry,
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
                    registry: &'a AssetRegistry,
                    manager: &'a mut AssetManager,
                }
                impl<'de, 'a> DeserializeSeed<'de> for BundlesDeserializeSeed<'a> {
                    type Value = ();
                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                        where D: Deserializer<'de> {
                        struct BundlesVisitor<'a> {
                            registry: &'a AssetRegistry,
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
                                    registry: &'a AssetRegistry,
                                }
                                impl<'de, 'a> DeserializeSeed<'de> for BundleDeserializeSeed<'a> {
                                    type Value = ImportAssetBundle;
                                    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
                                        where D: Deserializer<'de> {
                                        ImportAssetBundle::deserialize(self.registry, deserializer)
                                    }
                                }
                                while let Some(import) = seq.next_element_seed(BundleDeserializeSeed {registry: self.registry })? {
                                    self.manager.import_bundle(import).map_err(Error::custom)?;
                                }
                                Ok(())
                            }
                        }
                        self.manager.bundles.clear();
                        self.manager.containers.clear();
                        self.manager.defaults.clear();
                        deserializer.deserialize_seq(BundlesVisitor { registry: self.registry, manager: self.manager })
                    }
                }
                seq.next_element_seed(BundlesDeserializeSeed { registry: self.registry, manager: self.manager })?;
                self.manager.defaults = seq.next_element()?.expect("Expect defaults");
                for asset in self.manager.defaults.keys() {
                    if self.manager.containers.get(asset).is_none() {
                        return Err(de::Error::custom("Asset type not found for default"));
                    }
                }
                Ok(())
            }
        }
        deserializer.deserialize_tuple(2, AssetManagerVisitor { registry, manager: self })?;
        Ok(())
    }

    pub(crate) fn set_default<A: Asset>(&mut self, asset: UID, uid: UID) -> Result<()> {
        *self.defaults.get_mut(&asset).with_context(|| "Asset type not found")? = uid;
        Ok(())
    }

    pub(crate) fn get<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<&'_ A> {
        self.container::<A>(asset)?.0.get(&uid).map(|entry| &entry.asset)
            .with_context(|| "Asset not found")
    }

    pub(crate) fn get_or_default<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<&'_ A> {
        let container = self.container::<A>(asset)?;
        let default = self.defaults.get(&asset);
        container.0.get(&uid)
            .or_else(|| {
                default.and_then(|uid| {
                    container.0.get(uid)
                })
            })
            .map(|entry| &entry.asset)
            .with_context(|| "Asset not found and no default provided")
    }

    pub(crate) fn entry<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<&'_ AssetEntry<A>> {
        self.container::<A>(asset)?.0.get(&uid)
            .with_context(|| "Asset not found")
    }

    pub(crate) fn iter<A: Asset>(&self, asset: UID) -> Result<impl Iterator<Item = &AssetEntry<A>>> {
        Ok(self.container::<A>(asset)?.0.values())
    }

    pub(crate) fn add_bundle(&mut self, name: &str) -> Result<()> {
        let uid = UID::new(name);
        if self.bundles.contains_key(&uid) { return Err(anyhow!("Bundle already exists")); }
        self.bundles.insert(uid, AssetBundle::new(name));
        Ok(())
    }

    pub(crate) fn serialize_bundle<S: Serializer>(&self, uid: UID, serializer: S) -> Result<S::Ok, S::Error> {
        struct BundleSerialize<'a> {
            manager: &'a AssetManager,
            assets: &'a HashMap<UID, HashSet<UID>>,
        }
        impl<'a> Serialize for BundleSerialize<'a> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer {
                struct ContainerSerialize<'a> {
                    asset: UID,
                    container: &'a dyn AnyAssetContainer,
                    set: &'a HashSet<UID>,
                }
                impl<'a> Serialize for ContainerSerialize<'a> {
                    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                        where S: Serializer {
                        let mut tuple = serializer.serialize_tuple(2)?;
                        tuple.serialize_element(&self.asset)?;
                        tuple.serialize_element(&self.container.serialize_entries(self.set))?;
                        tuple.end()
                    }
                }
                let mut seq = serializer.serialize_seq(Some(self.assets.len()))?;
                for (asset, set) in self.assets {
                    let container = self.manager.containers.get(asset).with_context(|| "Asset container not found").map_err(Error::custom)?;
                    seq.serialize_element(&ContainerSerialize { asset: *asset, container: container.as_ref(), set })?;
                }
                seq.end()
            }
        }
        use serde::ser::Error;
        let bundle = self.bundles.get(&uid).with_context(|| "Bundle not found").map_err(S::Error::custom)?;
        let mut tuple = serializer.serialize_tuple(2)?;
        tuple.serialize_element(&bundle.name)?;
        tuple.serialize_element(&BundleSerialize { manager: self, assets: &bundle.assets })?;
        tuple.end()
    }

    pub(crate) fn import_bundle(&mut self, import: ImportAssetBundle) -> Result<()> {
        self.add_bundle(&import.name)?;
        for (asset, mut container) in import.containers {
            if let Some(self_container) = self.containers.get_mut(&asset) {
                self_container.merge(container.as_mut())?;
            } else {
                self.containers.insert(asset, container);
            }
        }
        Ok(())
    }

    pub(crate) fn add<A: Asset>(&mut self, registry: &AssetRegistry, asset: UID, name: &str, bundle: UID, data: A) -> Result<()> {
        // Check bundle
        if !self.bundles.contains_key(&bundle) { return Err(anyhow!("Bundle not found")); }
        // Get/Create the container
        let container = match self.containers.entry(asset) {
            hash_map::Entry::Occupied(entry) => {
                entry.into_mut()
            },
            hash_map::Entry::Vacant(entry) => {
                let definition = registry.get(asset).with_context(|| "Asset type not found")?;
                entry.insert(definition.reflection.create_container())
            },
        };
        // Downcast the container
        let container = container.as_any_mut().downcast_mut::<AssetContainer<A>>().with_context(|| "Asset type mismatch")?;
        // Check if asset already exists
        let uid = UID::new(name);
        if container.0.contains_key(&uid) { return Err(anyhow!("Asset '{}' already exists", name)); }
        // Safely insert the asset
        let value = AssetEntry { name: name.to_string(), asset: data, bundle };
        container.0.insert(uid, value);
        self.bundles.get_mut(&bundle).unwrap().assets.entry(asset)
            .or_insert_with(Default::default)
            .insert(uid);
        Ok(())
    }

    pub(crate) fn remove<A: Asset>(&mut self, asset: UID, uid: UID) -> Result<()> {
        // Get the container
        let container = self.containers.get_mut(&asset).with_context(|| "Asset type not found")?
            .as_any_mut().downcast_mut::<AssetContainer<A>>().with_context(|| "Asset type mismatch")?;
        // Remove the asset
        if let Some(entry) = container.0.remove(&uid) {
            self.bundles.get_mut(&entry.bundle).expect("Bundle not found")
                .assets.get_mut(&asset).expect("Asset not found")
                .remove(&uid);
        } else {
            return Err(anyhow!("Asset not found"));
        }
        Ok(())
    }

    pub(crate) fn transfer<A: Asset>(&mut self, asset: UID, uid: UID, dst_bundle: UID) -> Result<()> {
        let src_bundle = self.container::<A>(asset)?.0.get(&uid)
            .with_context(|| "Asset not found")?.bundle;
        if !self.bundles.contains_key(&dst_bundle) { return Err(anyhow!("Invalid destination bundle")); }
        if src_bundle == dst_bundle { return Ok(()); }
        self.bundles.get_mut(&src_bundle).with_context(|| "Source bundle not found")?
            .assets.get_mut(&asset).with_context(|| "Typeid in source bundle not found")?
            .remove(&uid);
        self.bundles.get_mut(&dst_bundle)
            .unwrap().assets.entry(asset).or_insert_with(Default::default).insert(uid);
        self.container_mut::<A>(asset)?.0.get_mut(&uid).unwrap().bundle = dst_bundle;
        Ok(())
    }
}