use std::collections::HashMap;

use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};
use slotmap::{SlotMap, Key};

use crate::app::App;
use crate::event::asset::ImportAssetEvent;

use self::bundle::Bundle;
use self::font::Font;
use self::input_action::InputAction;
use self::input_axis::InputAxis;
use self::input_table::InputTable;
use self::material::Material;
use self::mesh::Mesh;
use self::model::Model;
use self::rhai_script::RhaiScript;
use self::texture::Texture;

pub mod bundle;
pub mod font;
pub mod input_action;
pub mod input_axis;
pub mod input_table;
pub mod material;
pub mod mesh;
pub mod model;
pub mod rhai_script;
pub mod texture;

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AssetUID(u64);

impl AssetUID {
    pub fn new(name: &str) -> Self {
        Self(const_fnv1a_hash::fnv1a_hash_str_64(name))
    }
    pub fn uid(uid: u64) -> Self {
        Self(uid)
    }
    pub fn null() -> Self {
        Self(0)
    }
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

impl From<&str> for AssetUID {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl From<u64> for AssetUID {
    fn from(uid: u64) -> Self {
        Self::uid(uid)
    }
}

pub trait Asset {
    type Id: Key;
    fn typename() -> &'static str;
}

// impl<A: Asset + Serialize> Serialize for AssetTable<A> {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
//         where S: Serializer,
//     {
//         let mut seq = serializer.serialize_seq(Some(self.entries.len()))?;
//         for (_, entry) in &self.entries {
//             seq.serialize_element(entry)?;
//         }
//         seq.end()
//     }
// }

// struct AssetTableDeserializer<A: Asset> { maker: PhantomData<A> }

// impl<'de, A: Asset + Deserialize<'de>> Visitor<'de> for AssetTableDeserializer<A> {
//     type Value = AssetTable<A>;

//     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//         formatter.write_str("Asset sequence")
//     }

//     fn visit_seq<E>(self, mut seq: E) -> Result<Self::Value, E::Error>
//         where E: SeqAccess<'de> {
//         let mut registry = AssetTable::<A>::default();        
//         while let Some(mut value) = seq.next_element::<AssetEntry<A>>()? {
//             value.hash = const_fnv1a_hash::fnv1a_hash_str_32(&value.name);
//             if registry.entries.values().all(|e| e.hash != value.hash) {
//                 registry.entries.insert(value);
//             }
//         }
//         Ok(registry)
//     }
// }

// impl<'de, A: Asset + Deserialize<'de>> Deserialize<'de> for AssetTable<A> {
//     fn deserialize<D>(deserializer: D) -> Result<AssetTable<A>, D::Error>
//     where D: Deserializer<'de> {
//         deserializer.deserialize_seq(AssetTableDeserializer::<A> { maker: PhantomData::default() })
//     }
// }

#[derive(Serialize, Deserialize)]
pub struct AssetEntry<A: Asset> {
    pub name: String,
    pub asset: A,
    #[serde(skip)]
    pub uid: AssetUID,
    #[serde(skip)]
    pub id: A::Id,
}

pub struct AssetRegistry<A: Asset> {
    entries: SlotMap<A::Id, AssetEntry<A>>,
    uid_to_id: HashMap<AssetUID, A::Id>,
    default: AssetRef<A>,
}

impl<A: Asset> Default for AssetRegistry<A> {
    fn default() -> Self {
        Self { entries: Default::default(), uid_to_id: Default::default(), default: Default::default() }
    }
}

#[derive(Default)]
pub struct AssetManager {
    bundles: AssetRegistry<Bundle>,
    fonts: AssetRegistry<Font>,
    input_actions: AssetRegistry<InputAction>,
    input_axis: AssetRegistry<InputAxis>,
    input_tables: AssetRegistry<InputTable>,
    materials: AssetRegistry<Material>,
    meshes: AssetRegistry<Mesh>,
    models: AssetRegistry<Model>,
    rhai_scripts: AssetRegistry<RhaiScript>,
    textures: AssetRegistry<Texture>,
}

macro_rules! into_registry {
    ($asset:ty, $field:ident) => {
        impl AsRef<AssetRegistry<$asset>> for AssetManager {
            fn as_ref(&self) -> &AssetRegistry<$asset> {
                &self.$field
            }
        }
        impl AsMut<AssetRegistry<$asset>> for AssetManager {
            fn as_mut(&mut self) -> &mut AssetRegistry<$asset> {
                &mut self.$field
            }
        }
    };
}

into_registry!(Bundle, bundles);
into_registry!(Font, fonts);
into_registry!(InputAction, input_actions);
into_registry!(InputAxis, input_axis);
into_registry!(InputTable, input_tables);
into_registry!(Material, materials);
into_registry!(Mesh, meshes);
into_registry!(Model, models);
into_registry!(RhaiScript, rhai_scripts);
into_registry!(Texture, textures);

impl AssetManager {

    pub(crate) fn dispatch_event(&mut self, event: ImportAssetEvent) -> Result<()> {
        match event {
            ImportAssetEvent::Font(font) => {
                self.register(&font.name, font.data)
                    .context(format!("Failed to register imported font '{}'", font.name))?;
            },
            ImportAssetEvent::Material(material) => {
                self.register(&material.name, material.data)
                    .context(format!("Failed to register imported material '{}'", material.name))?;
            },
            ImportAssetEvent::Mesh(mesh) => {
                self.register(&mesh.name, mesh.data)
                    .context(format!("Failed to register imported mesh '{}'", mesh.name))?;
            },
            ImportAssetEvent::Model(model) => {
                self.register(&model.name, model.data)
                    .context(format!("Failed to register imported model '{}'", model.name))?;
            },
            ImportAssetEvent::RhaiScript(script) => {
                self.register(&script.name, script.data)
                    .context(format!("Failed to register imported lua script '{}'", script.name))?;
            },
            ImportAssetEvent::Texture(texture) => {
                self.register(&texture.name, texture.data)
                    .context(format!("Failed to register imported texture '{}'", texture.name))?;
            },
        }
        Ok(())
    }

    pub fn get<'a, A: Asset>(&'a self, id: A::Id) -> Option<&'a AssetEntry<A>>
        where Self: AsRef<AssetRegistry<A>> {
        let registry: &AssetRegistry<A> = self.as_ref();
        registry.entries.get(id)
    }

    pub fn find<'a, A: Asset>(&'a self, uid: AssetUID) -> Option<&'a AssetEntry<A>>
        where Self: AsRef<AssetRegistry<A>> {
        let registry: &AssetRegistry<A> = self.as_ref();
        registry.uid_to_id.get(&uid).and_then(|id| {
            registry.entries.get(*id)
        })
    }

    pub fn iter<'a, A: Asset + 'a>(&'a self) -> impl Iterator<Item = &'a AssetEntry<A>>
        where Self: AsRef<AssetRegistry<A>> {
        let registry: &AssetRegistry<A> = self.as_ref();
        registry.entries.values()
    }

    pub fn register<A: Asset>(&mut self, name: &str, data: A) -> Result<A::Id>
        where Self: AsMut<AssetRegistry<A>> {
        let registry: &mut AssetRegistry<A> = self.as_mut();
        let uid = AssetUID::new(name);
        let entry = AssetEntry { name: name.to_string(), asset: data, uid, id: A::Id::null() };
        if !registry.uid_to_id.contains_key(&uid) {
            let id = registry.entries.insert(entry);
            registry.entries.get_mut(id).unwrap().id = id;
            registry.uid_to_id.insert(uid, id);
            Ok(id)
        } else {
            Err(anyhow!("Asset already exists"))
        }
    }

    pub fn unregister<A: Asset>(&mut self, id: A::Id) -> Result<()>
        where Self: AsMut<AssetRegistry<A>> {
        let registry: &mut AssetRegistry<A> = self.as_mut();
        if registry.entries.contains_key(id) {

        } else {
            
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssetRef<A: Asset> {
    pub uid: AssetUID,
    #[serde(skip)]
    id: A::Id,
}

impl<A: Asset> Clone for AssetRef<A> {
    fn clone(&self) -> Self {
        Self { uid: self.uid, id: self.id }
    }
}

impl<A: Asset> PartialEq for AssetRef<A> {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }
}

impl<A: Asset> Default for AssetRef<A> {
    fn default() -> Self {
        Self::null()
    }
}

impl<A: Asset> From<&AssetEntry<A>> for AssetRef<A> {
    fn from(entry: &AssetEntry<A>) -> Self {
        Self { uid: entry.uid, id: entry.id }
    }
}

impl<A: Asset> From<&str> for AssetRef<A> {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

impl<'a, A: Asset> AssetRef<A> {

    pub fn null() -> Self {
        Self { uid: AssetUID::null(), id: A::Id::null() }
    }

    pub fn new(name: &str) -> Self {
        Self { uid: AssetUID::new(name), id: A::Id::null() }
    }

    pub fn uid(uid: AssetUID) -> Self {
        Self { uid, id: A::Id::null() }
    }

    pub fn resolve(&'a mut self, asset: &'a AssetManager) -> Result<()>
        where AssetManager: AsRef<AssetRegistry<A>> {
        let registry: &AssetRegistry<A> = asset.as_ref();
        if let Some(id) = registry.uid_to_id.get(&self.uid) {
            self.id = *id;
            Ok(())
        } else {
            self.id = A::Id::null();
            Err(anyhow!("Failed to resolve asset id"))
        }
    }

    pub fn get(&'a self, asset: &'a AssetManager) -> Option<&'a AssetEntry<A>>
        where AssetManager: AsRef<AssetRegistry<A>> {
        let registry: &AssetRegistry<A> = asset.as_ref();
        registry.entries.get(self.id)
    }

    pub fn get_or_default(&'a self, asset: &'a AssetManager) -> Option<&'a AssetEntry<A>>
        where AssetManager: AsRef<AssetRegistry<A>> {
        let registry: &AssetRegistry<A> = asset.as_ref();
        self.get(asset).or_else(|| {
            registry.default.get(asset)
        })
    }

    // pub fn get_or_resolve(&'a mut self, asset: &'a AssetManager) -> Option<&'a AssetEntry<A>>
    //     where AssetManager: AsRef<AssetRegistry<A>> {
    //     let registry: &AssetRegistry<A> = asset.as_ref();
    //     registry.entries.get(self.id).or_else(|| {
    //         if self.resolve(asset).is_ok() {
    //             registry.entries.get(self.id)
    //         } else {
    //             None
    //         }
    //     })
    // }

    pub fn is_null(&self) -> bool {
        self.uid.is_null()
    }
}

pub struct AssetDatabase;

impl AssetDatabase {
    pub fn get<'a, A: Asset>(app: &'a App, id: A::Id) -> Option<&'a AssetEntry<A>>
        where AssetManager: AsRef<AssetRegistry<A>> {
        app.asset_manager.get(id)
    }
    pub fn find<'a, A: Asset>(app: &'a App, uid: AssetUID) -> Option<&'a AssetEntry<A>>
        where AssetManager: AsRef<AssetRegistry<A>> {
        app.asset_manager.find(uid)
    }
}