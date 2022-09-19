use std::fmt::Display;

use slotmap::{SlotMap, SecondaryMap, new_key_type, Key};

use crate::event::asset::{AssetEvent, ImportAssetEvent};

use self::font::Font;
use self::material::Material;
use self::mesh::Mesh;
use self::texture::Texture;

pub mod font;
pub mod material;
pub mod mesh;
pub mod texture;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetName(String);

impl Default for AssetName {
    fn default() -> Self {
        Self("default".into())
    }
}

impl Display for AssetName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0.as_str())
    }
}

impl From<&str> for AssetName {
    fn from(name: &str) -> Self {
        Self(name.into())
    }
}

impl From<String> for AssetName {
    fn from(name: String) -> Self {
        Self(name)
    }
}

new_key_type! { 
    pub struct FontId;
    pub struct MaterialId;
    pub struct MeshId;
    pub struct TextureId;
}

pub trait Asset {
    fn typename() -> &'static str;
    fn default() -> Self;
}

pub struct AssetRegistry<K: Key, A: Asset> {
    assets: SlotMap<K, Box<A>>,
    names: SecondaryMap<K, AssetName>,
    default_id: K,
}

impl<K: Key, A: Asset> AssetRegistry<K, A> {
    pub(crate) fn register(&mut self, name: AssetName, data: Box<A>) -> K {
        let id = self.assets.insert(data);
        self.names.insert(id, name);
        id
    }

    pub fn get<'a>(&'a self, id: K) -> &'a A {
        &self.assets.get(id).unwrap_or(self.assets.get(self.default_id).expect(&format!("No default {}.", A::typename())))
    }

    pub fn get_from_name<'a>(&'a self, name: &str) -> &'a A {
        let id = self.names.iter().find(|(_, v)| v.0.as_str() == name).map_or(self.default_id, |(k, _)| k);
        &self.assets.get(id).unwrap()
    }

    pub fn default<'a>(&'a self) -> &'a A {
        self.get(self.default_id)
    }

    pub fn default_id(&self) -> K {
        self.default_id
    }
}

impl<K: Key, A: Asset> Default for AssetRegistry<K, A> {
    fn default() -> Self {
        let mut registry = Self { 
            assets: SlotMap::default(), 
            names: SecondaryMap::default(), 
            default_id: K::default()
        };
        registry.default_id = registry.register(AssetName::default(), Box::new(A::default()));
        registry
    }
}

#[derive(Default)]
pub struct AssetManager {
    pub fonts: AssetRegistry<FontId, Font>,
    pub materials: AssetRegistry<MaterialId, Material>,
    pub meshes: AssetRegistry<MeshId, Mesh>,
    pub textures: AssetRegistry<TextureId, Texture>,
}

impl AssetManager {
    pub(crate) fn dispatch_event(&mut self, event: AssetEvent) {
        match event {
            AssetEvent::Import(importer) => {
                match importer {
                    ImportAssetEvent::Font(_) => { },
                    ImportAssetEvent::Material(_) => { },
                    ImportAssetEvent::Mesh(mesh) => { 
                        self.meshes.register(mesh.name, mesh.data);
                    },
                    ImportAssetEvent::Texture(texture) => {
                        self.textures.register(texture.name, texture.data);
                    },
                }
            },
        }
    }
}