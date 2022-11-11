use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};

use crate::program::ProgramId;
use crate::uid::UID;

use self::font::Font;
use self::input_action::InputAction;
use self::input_axis::InputAxis;
use self::input_table::InputTable;
use self::material::Material;
use self::mesh::Mesh;
use self::model::Model;
use self::rhai_script::RhaiScript;
use self::system_schedule::SystemSchedule;
use self::texture::Texture;

pub mod font;
pub mod input_action;
pub mod input_axis;
pub mod input_table;
pub mod material;
pub mod mesh;
pub mod model;
pub mod rhai_script;
pub mod system_schedule;
pub mod texture;

pub trait Asset: Clone {}

#[derive(Clone, Serialize, Deserialize)]
pub struct AssetEntry<A: Asset> {
    #[serde(skip)]
    pub uid: UID,
    #[serde(skip)]
    pub bundle: UID,
    pub name: String,
    pub asset: A,
}

pub struct AssetRegistry<A: Asset> {
    entries: HashMap<UID, AssetEntry<A>>,
    default: Option<UID>,
}

impl<A: Asset> Default for AssetRegistry<A> {
    fn default() -> Self {
        Self { entries: Default::default(), default: Default::default() }
    }
}

macro_rules! reflect_database {
    (
        pub struct $database_type:ident {
            $(
                $field_name:ident : AssetRegistry<$asset_type:ty>,
            )*
        }
    ) => {
        #[derive(Default)]
        pub struct $database_type {
            $(
                $field_name : AssetRegistry<$asset_type>,
            )*
        }

        pub trait AssetRegistryRef<A: Asset> {
            fn as_registry(&self) -> &AssetRegistry<A>;
        }
        pub trait AssetRegistryMut<A: Asset> {
            fn as_registry_mut(&mut self) -> &mut AssetRegistry<A>;
        }
        pub trait UIDHashSetMut<A: Asset> {
            fn as_hashset_mut(&mut self) -> &mut HashSet<UID>;
        }

        $(
            impl AssetRegistryRef<$asset_type> for AssetManager {
                fn as_registry(&self) -> &AssetRegistry<$asset_type> {
                    &self.database.$field_name
                }
            }
            impl AssetRegistryMut<$asset_type> for AssetManager {
                fn as_registry_mut(&mut self) -> &mut AssetRegistry<$asset_type> {
                    &mut self.database.$field_name
                }
            }
            impl UIDHashSetMut<$asset_type> for AssetBundleInfo {
                fn as_hashset_mut(&mut self) -> &mut HashSet<UID> {
                    &mut self.$field_name
                }
            }
        )*

        #[derive(Default)]
        pub struct AssetBundleInfo {
            name: String,
            owner: ProgramId,
            $(
                $field_name : HashSet<UID>,
            )*
        }

        #[derive(Default, Serialize, Deserialize)]
        pub struct AssetBundleEntry<A> {
            name: String,
            asset: A,
        }

        #[derive(Default, Serialize, Deserialize)]
        pub struct AssetBundle {
            pub name: String,
            $(
                pub $field_name: Vec<AssetBundleEntry<$asset_type>>,
            )*
        }

        impl AssetManager {
            
            pub fn export_bundle(&self, bundle: UID) -> Result<AssetBundle> {
                let bundle = self.bundles.get(&bundle).context("Bundle not found")?;
                let mut export = AssetBundle { name: bundle.name.clone(), ..Default::default() };
                $(
                    for uid in &bundle.$field_name {
                        let entry = self.database.$field_name.entries.get(uid).context("Asset not found")?;
                        export.$field_name.push(AssetBundleEntry { name: entry.name.clone(), asset: entry.asset.clone() });
                    }
                )*
                Ok(export)
            }

            fn check_import_bundle(&self, bundle: &AssetBundle) -> Result<()> {
                let uid = UID::new(&bundle.name);
                if self.bundles.contains_key(&uid) { return Err(anyhow!("Bundle already exists")); }
                $(
                    for entry in &bundle.$field_name {
                        if self.database.$field_name.entries.contains_key(&UID::new(&entry.name)) {
                            return Err(anyhow!("Asset already exists"));
                        }
                    }
                )*
                Ok(())
            }

            pub fn import_bundle(&mut self, mut bundle: AssetBundle, owner: ProgramId) -> Result<()> {
                self.check_import_bundle(&bundle)?;
                let uid = UID::new(&bundle.name);
                let mut new_bundle = AssetBundleInfo { name: bundle.name.clone(), owner, ..Default::default() };
                $(
                    for entry in &bundle.$field_name {
                        new_bundle.$field_name.insert(UID::new(&entry.name));
                    }
                )*
                self.bundles.insert(uid, new_bundle);
                $(
                    for entry in bundle.$field_name.drain(..) {
                        self.register::<$asset_type>(&entry.name, uid, entry.asset)?;
                    }
                )*
                Ok(())
            }
        }
    };
}

reflect_database!(
    pub struct AssetDatabase {
        fonts: AssetRegistry<Font>,
        input_actions: AssetRegistry<InputAction>,
        input_axis: AssetRegistry<InputAxis>,
        input_tables: AssetRegistry<InputTable>,
        materials: AssetRegistry<Material>,
        meshes: AssetRegistry<Mesh>,
        models: AssetRegistry<Model>,
        rhai_scripts: AssetRegistry<RhaiScript>,
        system_schedules: AssetRegistry<SystemSchedule>,
        textures: AssetRegistry<Texture>,
    }
);

#[derive(Default)]
pub struct AssetManager {
    database: AssetDatabase,
    bundles: HashMap<UID, AssetBundleInfo>,
}

impl AssetManager {

    pub fn bundle(&'_ self, uid: UID) -> Option<&'_ AssetBundleInfo> {
        self.bundles.get(&uid)
    }

    pub fn set_default<A: Asset>(&mut self, uid: UID) -> Result<()>
        where Self: AssetRegistryMut<A> {
        self.as_registry_mut().default = Some(uid);
        Ok(())
    }

    pub fn get<A: Asset>(&'_ self, uid: UID) -> Option<&'_ A>
        where Self: AssetRegistryRef<A> {
        self.as_registry().entries.get(&uid).map(|entry| &entry.asset)
    }

    pub fn get_or_default<A: Asset>(&'_ self, uid: UID) -> Option<&'_ A>
        where Self: AssetRegistryRef<A> {
        self.as_registry().entries.get(&uid)
        .or_else(|| {
            self.as_registry().default.and_then(|uid| {
                self.as_registry().entries.get(&uid)
            })
        })
        .map(|entry| &entry.asset)
    }

    pub fn get_mut<A: Asset>(&'_ mut self, uid: UID) -> Option<&'_ mut A> 
        where Self: AssetRegistryMut<A> {
        self.as_registry_mut().entries.get_mut(&uid).map(|entry| &mut entry.asset)
    }

    pub fn entry<A: Asset>(&'_ self, uid: UID) -> Option<&'_ AssetEntry<A>>
        where Self: AssetRegistryRef<A> {
        self.as_registry().entries.get(&uid)
    }

    pub fn iter<'a, A: Asset + 'a>(&'a self) -> impl Iterator<Item = &'a AssetEntry<A>>
        where Self: AssetRegistryRef<A> {
        self.as_registry().entries.values()
    }

    pub fn register_bundle(&mut self, name: &str, owner: ProgramId) -> Result<()> {
        let uid = UID::new(name);
        if self.bundles.contains_key(&uid) { return Err(anyhow!("Bundle already exists")); }
        let bundle = AssetBundleInfo { name: name.to_string(), owner, ..Default::default() };
        self.bundles.insert(uid, bundle);
        Ok(())
    }

    pub fn register<A: Asset>(&mut self, name: &str, bundle: UID, data: A) -> Result<()>
        where Self: AssetRegistryMut<A> + AssetRegistryRef<A>, AssetBundleInfo: UIDHashSetMut<A> {
        let uid = UID::new(name);
        let value = AssetEntry { uid, bundle, name: name.to_string(), asset: data };
        if !self.bundles.contains_key(&bundle) { return Err(anyhow!("Bundle not found")); }
        if self.as_registry().entries.contains_key(&uid) { return Err(anyhow!("Asset '{}' already exists", name)); }
        self.as_registry_mut().entries.insert(uid, value);
        self.bundles.get_mut(&bundle).unwrap().as_hashset_mut().insert(uid);
        Ok(())
    }

    pub fn unregister<A: Asset>(&mut self, uid: UID) -> Result<()>
        where Self: AssetRegistryMut<A>, AssetBundleInfo: UIDHashSetMut<A> {
        if !self.as_registry_mut().entries.contains_key(&uid) { return Err(anyhow!("Asset not found")); }
        {
            // Remove from bundle
            let bundle_uid = self.as_registry_mut().entries.get(&uid).unwrap().bundle;
            let bundle = self.bundles.get_mut(&bundle_uid).context("Bundle not found")?;
            bundle.as_hashset_mut().remove(&uid);
        }
        {
            // TODO: check dependencies
            self.as_registry_mut().entries.remove(&uid);
        }
        Ok(())
    }

    pub fn transfer<A: Asset>(&mut self, uid: UID, dst_bundle: UID) -> Result<()> 
        where Self: AssetRegistryRef<A> + AssetRegistryMut<A>, AssetBundleInfo: UIDHashSetMut<A> {
        let src_bundle = self.as_registry().entries.get(&uid)
            .context("Asset not found")?.bundle;
        if !self.bundles.contains_key(&dst_bundle) { return Err(anyhow!("Invalid destination bundle")); }
        if src_bundle == dst_bundle { return Ok(()); }
        self.bundles.get_mut(&src_bundle)
            .context("Source bundle not found")?.as_hashset_mut().remove(&uid);
        self.bundles.get_mut(&dst_bundle)
            .unwrap().as_hashset_mut().insert(uid);
        self.as_registry_mut().entries.get_mut(&uid).unwrap().bundle = dst_bundle;
        Ok(())
    }
}