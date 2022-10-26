use std::fmt;
use std::marker::PhantomData;

use anyhow::{Result, anyhow, Context};
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use slotmap::{SlotMap, Key, new_key_type};

use crate::event::asset::ImportAssetEvent;
use crate::program::ProgramId;

use self::font::Font;
use self::material::Material;
use self::mesh::Mesh;
use self::model::Model;
use self::rhai_script::RhaiScript;
use self::texture::Texture;

pub mod font;
pub mod material;
pub mod mesh;
pub mod model;
pub mod rhai_script;
pub mod texture;

new_key_type! { pub struct AssetBundleId; }
new_key_type! { pub struct AssetId; }

pub trait Asset {
    const IMPORT_BUNDLE: &'static str = "import";
    fn typename() -> &'static str;
}

#[derive(Serialize, Deserialize)]
pub struct AssetEntry<A: Asset> {
    name: String,
    data: A,
    #[serde(skip)]
    hash: u32,
}

pub struct AssetTable<A: Asset> {
    pub(crate) entries: SlotMap<AssetId, AssetEntry<A>>,
}

impl<A: Asset> Default for AssetTable<A> {
    fn default() -> Self {
        Self { entries: SlotMap::with_key() }
    }
}

impl<A: Asset + Serialize> Serialize for AssetTable<A> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
        where S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.entries.len()))?;
        for (_, entry) in &self.entries {
            seq.serialize_element(entry)?;
        }
        seq.end()
    }
}

struct AssetTableDeserializer<A: Asset> { maker: PhantomData<A> }

impl<'de, A: Asset + Deserialize<'de>> Visitor<'de> for AssetTableDeserializer<A> {
    type Value = AssetTable<A>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Asset sequence")
    }

    fn visit_seq<E>(self, mut seq: E) -> Result<Self::Value, E::Error>
        where E: SeqAccess<'de> {
        let mut registry = AssetTable::<A>::default();        
        while let Some(mut value) = seq.next_element::<AssetEntry<A>>()? {
            value.hash = const_fnv1a_hash::fnv1a_hash_str_32(&value.name);
            if registry.entries.values().all(|e| e.hash != value.hash) {
                registry.entries.insert(value);
            }
        }
        Ok(registry)
    }
}

impl<'de, A: Asset + Deserialize<'de>> Deserialize<'de> for AssetTable<A> {
    fn deserialize<D>(deserializer: D) -> Result<AssetTable<A>, D::Error>
    where D: Deserializer<'de> {
        deserializer.deserialize_seq(AssetTableDeserializer::<A> { maker: PhantomData::default() })
    }
}

impl<'a, A: Asset> AssetTable<A> {

    fn add(&mut self, name: &str, data: A) -> Result<AssetId> {
        let hash = const_fnv1a_hash::fnv1a_hash_str_32(name);
        if self.entries.iter().any(|(_, e)| e.hash == hash) {
            return Err(anyhow!("Duplicated asset name '{}'", name));
        }
        let id = self.entries.insert(AssetEntry { 
            name: name.to_string(),
            hash,
            data,
        });
        Ok(id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetBundle {

    // Meta data
    name: String,
    #[serde(skip)]
    hash: u32,
    #[serde(skip)]
    owner: ProgramId,
    #[serde(skip)]
    read_only: bool,

    // Tables
    fonts: AssetTable<Font>,
    materials: AssetTable<Material>,
    meshes: AssetTable<Mesh>,
    models: AssetTable<Model>,
    rhai_scripts: AssetTable<RhaiScript>,
    textures: AssetTable<Texture>,
}

macro_rules! into_table {
    ($asset:ty, $field:ident) => {
        impl AsRef<AssetTable<$asset>> for AssetBundle {
            fn as_ref(&self) -> &AssetTable<$asset> {
                &self.$field
            }
        }
        impl AsMut<AssetTable<$asset>> for AssetBundle {
            fn as_mut(&mut self) -> &mut AssetTable<$asset> {
                &mut self.$field
            }
        }
    };
}

into_table!(Font, fonts);
into_table!(Material, materials);
into_table!(Mesh, meshes);
into_table!(Model, models);
into_table!(RhaiScript, rhai_scripts);
into_table!(Texture, textures);

impl AssetBundle {

    pub const IMPORT: &'static str = "import";
    pub const DEFAULT: &'static str = "default";

    fn new(name: &str, owner: ProgramId) -> Self {
        Self {
            name: name.to_string(),
            hash: const_fnv1a_hash::fnv1a_hash_str_32(name),
            owner,
            read_only: false,
            fonts: Default::default(),
            materials: Default::default(),
            meshes: Default::default(),
            models: Default::default(),
            rhai_scripts: Default::default(),
            textures: Default::default(),
        }
    }

    pub fn name<'a>(&'a self) -> &'a str {
        &self.name
    }

    pub fn owner(&self) -> ProgramId {
        self.owner
    }

    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    pub fn register<A: Asset>(&mut self, name: &str, data: A) -> Result<AssetId>
        where Self: AsMut<AssetTable<A>> {
        if self.read_only {
            return Err(anyhow!("Asset bundle is read-only"));
        }
        let table: &mut AssetTable<A> = self.as_mut();
        table.add(name, data)
    }

    pub fn unregister<A: Asset>(&mut self, id: AssetId) -> Result<()>
        where Self: AsMut<AssetTable<A>> {
        if self.read_only {
            return Err(anyhow!("Asset bundle is read-only"));
        }
        let table: &mut AssetTable<A> = self.as_mut();
        if table.entries.remove(id).is_some() {
            Ok(())
        } else {
            Err(anyhow!("Asset not found"))
        }
    } 

    pub fn find_from_hash<A: Asset>(&self, hash: u32) -> Option<AssetId> where Self: AsRef<AssetTable<A>> {
        let table: &AssetTable<A> = self.as_ref();
        table.entries.iter()
            .find(|(_, entry)| entry.hash == hash)
            .map(|(id, _)| id)
    }

    pub fn find<A: Asset>(&self, name: &str) -> Option<AssetId> where Self: AsRef<AssetTable<A>> {
        self.find_from_hash(const_fnv1a_hash::fnv1a_hash_str_32(name))
    }

    pub fn get<'a, A: Asset>(&'a self, id: AssetId) -> Option<&'a AssetEntry<A>> where Self: AsRef<AssetTable<A>> {
        let table: &AssetTable<A> = self.as_ref();
        table.entries.get(id)
    }

    pub fn iter_ids<'a, A: Asset + 'a>(&'a self) -> impl Iterator<Item = AssetId> + 'a where Self: AsRef<AssetTable<A>> {
        let table: &AssetTable<A> = self.as_ref();
        table.entries.iter().map(|(id, _)| id)
    }

    pub fn iter<'a, A: Asset + 'a>(&'a self) -> impl Iterator<Item = &'a A> + 'a where Self: AsRef<AssetTable<A>> {
        let table: &AssetTable<A> = self.as_ref();
        table.entries.iter().map(|(_, e)| &e.data)
    }
}

pub struct AssetManager {
    bundles: SlotMap<AssetBundleId, AssetBundle>,
    import_bundle: AssetBundleId,
    default_bundle: AssetBundleId,
}

impl Default for AssetManager {

    fn default() -> Self {
        // Default manager
        let mut manager = Self { bundles: Default::default(), import_bundle: AssetBundleId::null(), default_bundle: AssetBundleId::null() };
        // Create bundles
        manager.import_bundle = manager.add_bundle(AssetBundle::IMPORT, ProgramId::null())
            .expect("Failed to add import bundle");
        manager.default_bundle = manager.add_bundle(AssetBundle::DEFAULT, ProgramId::null())
            .expect("Failed to add default bundle");
        // Register default assets
        manager.bundle_mut(manager.default_bundle).unwrap().register::<Font>("default", Font::default())
            .expect("Failed to register default font");
        // Return manager
        manager
    }
}

impl AssetManager {

    pub(crate) fn dispatch_event(&mut self, event: ImportAssetEvent) -> Result<()> {
        match event {
            ImportAssetEvent::Font(font) => {
                self.bundle_mut(self.import_bundle).unwrap().register(&font.name, font.data)
                    .context(format!("Failed to register imported font '{}'", font.name))?;
            },
            ImportAssetEvent::Material(material) => {
                self.bundle_mut(self.import_bundle).unwrap().register(&material.name, material.data)
                    .context(format!("Failed to register imported material '{}'", material.name))?;
            },
            ImportAssetEvent::Mesh(mesh) => {
                self.bundle_mut(self.import_bundle).unwrap().register(&mesh.name, mesh.data)
                    .context(format!("Failed to register imported mesh '{}'", mesh.name))?;
            },
            ImportAssetEvent::Model(model) => {
                self.bundle_mut(self.import_bundle).unwrap().register(&model.name, model.data)
                    .context(format!("Failed to register imported model '{}'", model.name))?;
            },
            ImportAssetEvent::RhaiScript(script) => {
                self.bundle_mut(self.import_bundle).unwrap().register(&script.name, script.data)
                    .context(format!("Failed to register imported lua script '{}'", script.name))?;
            },
            ImportAssetEvent::Texture(texture) => {
                self.bundle_mut(self.import_bundle).unwrap().register(&texture.name, texture.data)
                    .context(format!("Failed to register imported texture '{}'", texture.name))?;
            },
        }
        Ok(())
    }

    pub fn transfer<A: Asset>(
        &mut self,
        src_bundle_id: AssetBundleId,
        src_asset_id: AssetId,
        dst_bundle_id: AssetBundleId,
    ) -> Result<()> where AssetBundle: AsMut<AssetTable<A>>, AssetBundle: AsRef<AssetTable<A>> {
        if src_bundle_id == dst_bundle_id { return Ok(()); }
        let src_bundle = self.bundle_mut(src_bundle_id)
            .context(anyhow!("Source bundle not found"))?;
        let src_table: &mut AssetTable<A> = src_bundle.as_mut();
        if let Some(entry) = src_table.entries.remove(src_asset_id) {
            self.bundle_mut(dst_bundle_id)
                .context(anyhow!("Destination bundle not found"))
                .and_then(|bundle| {
                    bundle.find::<A>(&entry.name).or_else(|| {
                        let dst_table: &mut AssetTable<A> = bundle.as_mut();
                        dst_table.add(&entry.name, entry.data).ok()
                    })
                    .context(anyhow!("Destination asset name already exists"))                
                })
                .context("")?;
                Ok(())
        } else {
            return Err(anyhow!("Source asset not found"));        
        }
    }

    pub fn bundle<'a>(&'a self, id: AssetBundleId) -> Option<&'a AssetBundle> {
        self.bundles.get(id)
    }

    pub fn bundle_mut<'a>(&'a mut self, id: AssetBundleId) -> Option<&'a mut AssetBundle> {
        self.bundles.get_mut(id)
    }

    pub fn import_bundle(&self) -> AssetBundleId {
        self.import_bundle
    }
    
    pub fn find_bundle_from_hash(&self, hash: u32) -> Option<AssetBundleId> {
        self.bundles.iter()
            .find(|(_, b)| b.hash == hash)
            .and_then(|(id, _)| Some(id))
    }

    pub fn find_bundle(&self, name: &str) -> Option<AssetBundleId> {
        let hash = const_fnv1a_hash::fnv1a_hash_str_32(name);
        self.find_bundle_from_hash(hash)
    }

    pub fn add_bundle(&mut self, name: &str, owner: ProgramId) -> Result<AssetBundleId> {
        let hash = const_fnv1a_hash::fnv1a_hash_str_32(name);
        if self.find_bundle_from_hash(hash).is_some() {
            return Err(anyhow!("Asset bundle '{}' already exists", name));
        }
        let id = self.bundles.insert(AssetBundle::new(name, owner));
        Ok(id)
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetRef<A: Asset> {
    pub bundle_hash: u32,
    pub asset_hash: u32,
    #[serde(skip)]
    pub bundle_id: AssetBundleId,
    #[serde(skip)]
    pub asset_id: AssetId,
    #[serde(skip)]
    marker: PhantomData<A>,
}

impl<A: Asset> Default for AssetRef<A> {
    fn default() -> Self {
        Self { 
            bundle_hash: 0, 
            asset_hash: 0, 
            bundle_id: AssetBundleId::null(), 
            asset_id: AssetId::null(), 
            marker: PhantomData
        }
    }
}

impl<A: Asset> Clone for AssetRef<A> {
    fn clone(&self) -> Self {
        Self { 
            bundle_hash: self.bundle_hash, 
            asset_hash: self.asset_hash, 
            bundle_id: self.bundle_id, 
            asset_id: self.asset_id, 
            marker: PhantomData 
        }
    }
}

impl<A: Asset> PartialEq for AssetRef<A> {
    fn eq(&self, other: &Self) -> bool {
        self.bundle_hash == other.bundle_hash && self.asset_hash == other.asset_hash
    }
}

pub type AssetUID = u64;

impl<'a, A: Asset + 'a> AssetRef<A> {

    pub fn new(bundle: &str, asset: &str) -> Self {
        Self {
            bundle_hash: const_fnv1a_hash::fnv1a_hash_str_32(bundle),
            asset_hash: const_fnv1a_hash::fnv1a_hash_str_32(asset),
            bundle_id: AssetBundleId::null(),
            asset_id: AssetId::null(),
            marker: PhantomData,
        }
    }

    fn resolve(&mut self, manager: &'a AssetManager) -> Option<&'a AssetEntry<A>>
        where AssetBundle: AsRef<AssetTable<A>> {
        manager.bundle(self.bundle_id).or_else(|| {
            manager.find_bundle_from_hash(self.bundle_hash).and_then(|id| {
                self.bundle_id = id;
                Some(manager.bundle(id).unwrap())
            })
        }).and_then(|bundle| {
            bundle.get::<A>(self.asset_id).or_else(|| {
                bundle.find_from_hash(self.asset_hash).and_then(|id| {
                    self.asset_id = id;
                    Some(bundle.get::<A>(id).unwrap())
                })
            })
        })
    }

    pub fn get(&mut self, manager: &'a AssetManager) -> Option<&'a A>
        where AssetBundle: AsRef<AssetTable<A>> {
        self.resolve(manager).and_then(|entry| Some(&entry.data))
    }

    pub fn name(&mut self, manager: &'a AssetManager) -> Option<&'a str>
        where AssetBundle: AsRef<AssetTable<A>> {
        self.resolve(manager).and_then(|entry| Some(entry.name.as_str()))
    }

    pub fn uid(&self) -> AssetUID {
        (self.bundle_hash as u64) << 32 | self.asset_hash as u64
    }
}

pub struct AssetDatabase;

impl AssetDatabase {
    // pub fn read<A: Asset>(app: &App, group: AssetGroupId, asset: A::Id) -> Option<&AssetEntry<A>> {
    //     app.asset_manager.get(group, asset)
    // }
}