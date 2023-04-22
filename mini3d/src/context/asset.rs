use std::cell::RefCell;

use crate::{asset::{AssetManager, AssetEntry, AssetError}, uid::UID, registry::{RegistryManager, asset::Asset}};

pub struct AssetContext<'a> {
    pub(crate) registry: &'a RefCell<RegistryManager>,
    pub(crate) manager: &'a mut AssetManager,
}

impl<'a> AssetContext<'a> {

    pub fn set_default<A: Asset>(&mut self, asset: UID, uid: UID) -> Result<(), AssetError> {
        self.manager.set_default::<A>(asset, uid)
    }

    pub fn get<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<Option<&'_ A>, AssetError> {
        self.manager.get::<A>(asset, uid)
    }

    pub fn get_or_default<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<Option<&'_ A>, AssetError> {
        self.manager.get_or_default::<A>(asset, uid)
    }

    pub fn entry<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<Option<&'_ AssetEntry<A>>, AssetError> {
        self.manager.entry::<A>(asset, uid)
    }

    pub fn iter<A: Asset>(&self, asset: UID) -> Result<Option<impl Iterator<Item = &AssetEntry<A>>>, AssetError> {
        self.manager.iter::<A>(asset)
    }

    pub fn add_bundle(&mut self, name: &str) -> Result<UID, AssetError> {
        self.manager.add_bundle(name)
    }

    pub fn add<A: Asset>(&mut self, asset: UID, name: &str, bundle: UID, data: A) -> Result<(), AssetError> {
        self.manager.add::<A>(&self.registry.borrow().assets, asset, name, bundle, data)
    }

    pub fn remove<A: Asset>(&mut self, asset: UID, uid: UID) -> Result<(), AssetError> {
        self.manager.remove::<A>(asset, uid)
    }

    pub fn transfer<A: Asset>(&mut self, asset: UID, uid: UID, dst_bundle: UID) -> Result<(), AssetError> {
        self.manager.transfer::<A>(asset, uid, dst_bundle)
    }
}