use std::fmt::Display;

use slotmap::{SlotMap, SecondaryMap, new_key_type};

use crate::event::asset::{AssetEvent, ImportAssetEvent};

use self::font::Font;
use self::material::Material;
use self::mesh::Mesh;
use self::texture::Texture;

pub mod font;
pub mod material;
pub mod mesh;
pub mod texture;

#[derive(Clone, Debug)]
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

new_key_type! { pub struct AssetId; }

pub trait Asset {
    fn typename() -> &'static str;
    fn default() -> Self;
}

pub struct AssetRegistry<T: Asset> {
    assets: SlotMap<AssetId, Box<T>>,
    names: SecondaryMap<AssetId, AssetName>,
    default_id: AssetId,
}

impl<T: Asset> AssetRegistry<T> {
    pub(crate) fn register(&mut self, name: AssetName, data: Box<T>) -> AssetId {
        let id = self.assets.insert(data);
        self.names.insert(id, name);
        id
    }

    pub fn get<'a>(&'a self, id: AssetId) -> &'a T {
        &self.assets.get(id).unwrap_or(self.assets.get(self.default_id).expect(&format!("No default {}.", T::typename())))
    }

    pub fn default<'a>(&'a self) -> &'a T {
        self.get(self.default_id)
    }
}

impl<T: Asset> Default for AssetRegistry<T> {
    fn default() -> Self {
        let mut registry = Self { 
            assets: SlotMap::default(), 
            names: SecondaryMap::default(), 
            default_id: AssetId::default()
        };
        registry.default_id = registry.register(AssetName::default(), Box::new(T::default()));
        registry
    }
}

#[derive(Default)]
pub struct AssetManager {
    pub fonts: AssetRegistry<Font>,
    pub materials: AssetRegistry<Material>,
    pub meshes: AssetRegistry<Mesh>,
    pub textures: AssetRegistry<Texture>,
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