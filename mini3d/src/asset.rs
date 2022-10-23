use anyhow::{Result, anyhow, Context};
use slotmap::{SlotMap, Key, new_key_type};

use crate::app::App;
use crate::event::asset::ImportAssetEvent;
use crate::program::ProgramId;

use self::font::Font;
use self::material::Material;
use self::mesh::Mesh;
use self::model::Model;
use self::script::RhaiScript;
use self::texture::Texture;

pub mod font;
pub mod material;
pub mod mesh;
pub mod model;
pub mod script;
pub mod texture;

new_key_type! { pub struct AssetGroupId; }

pub trait Asset {
    type Id: Key;
    fn typename() -> &'static str;
}

pub struct AssetEntry<A: Asset> {
    pub data: A,
    pub name: String,
    pub id: A::Id,
    pub group: AssetGroupId,
}

pub struct AssetRegistry<A: Asset> {
    entries: SlotMap<A::Id, AssetEntry<A>>,
    default_id: Option<A::Id>,
}

impl<A: Asset> Default for AssetRegistry<A> {
    fn default() -> Self {
        Self {
            entries: SlotMap::with_key(),
            default_id: None,
        }
    }
}

impl<'a, A: Asset> AssetRegistry<A> {

    pub fn register(&mut self, name: &str, group: AssetGroupId, data: A) -> Result<A::Id> {
        if self.entries.iter().any(|(_, e)| e.name == name) {
            Err(anyhow!("Asset name '{}' already exists", name))
        } else {
            let id = self.entries.insert(AssetEntry { 
                data,
                name: name.to_string(), 
                id: A::Id::default(),
                group,
            });
            self.entries.get_mut(id).unwrap().id = id;
            Ok(id)
        }
    }
    
    pub fn get(&self, id: A::Id) -> Option<&AssetEntry<A>> {
        self.entries.get(id)
    }
    
    pub fn default(&self) -> Option<&AssetEntry<A>> {
        self.default_id.and_then(|id| self.get(id))
    }
    
    pub fn set_default(&mut self, id: A::Id) -> Result<()> {
        if !self.entries.contains_key(id) {
            Err(anyhow!("Trying to set default asset with invalid id"))
        } else {
            self.default_id = Some(id);
            Ok(())
        }
    }
    
    pub fn find(&self, name: &str, group: AssetGroupId) -> Option<&AssetEntry<A>> {
        self.entries.iter()
            .find(|(_, e)| e.name == name && e.group == group)
            .map(|(_, e)| e)
    }

    pub fn iter(&'a self) -> impl Iterator<Item = &'a AssetEntry<A>> {
        self.entries.values()
    }
    
    pub fn iter_group(&self, group: AssetGroupId) -> impl Iterator<Item = &AssetEntry<A>> {
        self.entries.values().filter(move |e| e.group == group)
    }

    pub fn transfer(&mut self, id: A::Id, new_group: AssetGroupId) -> Result<()> {
        if let Some(e) = self.entries.get_mut(id) {
            e.group = new_group;
            Ok(())
        } else {
            Err(anyhow!("Failed to transfer <{}> because id was not found", A::typename()))
        }
    }
}

pub struct AssetGroup {
    pub name: String,
    pub id: AssetGroupId,
    pub owner: ProgramId,
} 

pub struct AssetManager {

    // Assets
    fonts: AssetRegistry<Font>,
    materials: AssetRegistry<Material>,
    meshes: AssetRegistry<Mesh>,
    models: AssetRegistry<Model>,
    rhai_scripts: AssetRegistry<RhaiScript>,
    textures: AssetRegistry<Texture>,    

    // Groups
    groups: SlotMap<AssetGroupId, AssetGroup>,
    import_group: AssetGroupId,
}

impl Default for AssetManager {
    fn default() -> Self {
        // Default manager
        let mut manager = Self { 
            fonts: Default::default(), 
            materials: Default::default(), 
            meshes: Default::default(), 
            models: Default::default(), 
            rhai_scripts: Default::default(),
            textures: Default::default(), 
            groups: Default::default(), 
            import_group: Default::default() 
        };
        // Register import group
        manager.import_group = manager.register_group("import", ProgramId::null())
            .expect("Failed to register import group");
        // Return manager
        manager
    }
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

into_registry!(Font, fonts);
into_registry!(Material, materials);
into_registry!(Mesh, meshes);
into_registry!(Model, models);
into_registry!(RhaiScript, rhai_scripts);
into_registry!(Texture, textures);

impl AssetManager {

    pub(crate) fn dispatch_event(&mut self, event: ImportAssetEvent) -> Result<()> {
        match event {
            ImportAssetEvent::Font(font) => {
                self.register(&font.name, self.import_group, font.data)
                    .context(format!("Failed to register imported font '{}'", font.name))?;
            },
            ImportAssetEvent::Material(material) => {
                self.register(&material.name, self.import_group, material.data)
                    .context(format!("Failed to register imported material '{}'", material.name))?;
            },
            ImportAssetEvent::Mesh(mesh) => {
                self.register(&mesh.name, self.import_group, mesh.data)
                    .context(format!("Failed to register imported mesh '{}'", mesh.name))?;
            },
            ImportAssetEvent::Model(model) => {
                self.register(&model.name, self.import_group, model.data)
                    .context(format!("Failed to register imported model '{}'", model.name))?;
            },
            ImportAssetEvent::RhaiScript(script) => {
                self.register(&script.name, self.import_group, script.data)
                    .context(format!("Failed to register imported lua script '{}'", script.name))?;
            },
            ImportAssetEvent::Texture(texture) => {
                self.register(&texture.name, self.import_group, texture.data)
                    .context(format!("Failed to register imported texture '{}'", texture.name))?;
            },
        }
        Ok(())
    }

    pub fn group(&self, id: AssetGroupId) -> Option<&AssetGroup> {
        self.groups.get(id)
    }
    
    pub fn find_group(&self, name: &str) -> Option<&AssetGroup> {
        self.groups.iter()
            .find(|(_, e)| e.name.as_str() == name)
            .and_then(|(_, group)| Some(group))
    }

    pub fn register_group(&mut self, name: &str, owner: ProgramId) -> Result<AssetGroupId> {
        if self.find_group(&name).is_some() {
            Err(anyhow!("Asset group '{}' already exists", name))
        } else {
            let new_group = self.groups.insert(AssetGroup { 
                name: name.to_string(), 
                id: AssetGroupId::null(),
                owner,
            });
            self.groups.get_mut(new_group).unwrap().id = new_group;
            Ok(new_group)
        }
    }

    pub fn register<'a, A: Asset>(&mut self, name: &str, group: AssetGroupId, data: A) -> Result<A::Id> 
        where Self: AsMut<AssetRegistry<A>> {
        if !self.groups.contains_key(group) {
            return Err(anyhow!("Trying to register asset with invalid group id"));
        }
        self.as_mut().register(name, group, data)
    }
    
    pub fn get<'a, A: Asset>(&self, id: A::Id) -> Option<&AssetEntry<A>> 
        where Self: AsRef<AssetRegistry<A>> {
        self.as_ref().get(id)
    }
    
    pub fn default<'a, A: Asset>(&self) -> Option<&AssetEntry<A>>
        where Self: AsRef<AssetRegistry<A>> {
        self.as_ref().default()
    }
    
    pub fn set_default<'a, A: Asset>(&mut self, id: A::Id) -> Result<()>
        where Self: AsMut<AssetRegistry<A>> {
        self.as_mut().set_default(id)
    }
    
    pub fn find<'a, A: Asset>(&self, name: &str, group: AssetGroupId) -> Option<&AssetEntry<A>>
        where Self: AsRef<AssetRegistry<A>> {
        self.as_ref().find(name, group)
    }

    pub fn iter<'a, A: Asset + 'a>(&'a self) -> impl Iterator<Item = &AssetEntry<A>>
        where Self: AsRef<AssetRegistry<A>> {
        self.as_ref().iter()
    }
    
    pub fn iter_group<'a, A: Asset + 'a>(&'a self, group: AssetGroupId) -> impl Iterator<Item = &AssetEntry<A>>
        where Self: AsRef<AssetRegistry<A>> {
        self.as_ref().iter_group(group)
    }

    pub fn iter_import<'a, A: Asset + 'a>(&'a self) -> impl Iterator<Item = &AssetEntry<A>> 
        where Self: AsRef<AssetRegistry<A>> {
        self.iter_group(self.import_group)
    }

    pub fn transfer<'a, A: Asset>(&mut self, id: A::Id, new_group: AssetGroupId) -> Result<()>
        where Self: AsMut<AssetRegistry<A>> {
        if !self.groups.contains_key(new_group) {
            return Err(anyhow!("Trying to transfer asset with invalid group id"));
        }
        self.as_mut().transfer(id, new_group)
    }    
}

pub struct AssetDatabase;

impl AssetDatabase {
    pub fn read<A: Asset>(app: &App, id: A::Id) -> Option<&AssetEntry<A>> 
        where AssetManager: AsRef<AssetRegistry<A>> {
        app.asset_manager.get(id)
    }
}