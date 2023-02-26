use std::cell::RefCell;
use anyhow::Result;

use crate::{asset::{AssetManager, AssetEntry}, uid::UID, registry::{RegistryManager, asset::Asset}};

pub struct AssetContext<'a> {
    pub(crate) registry: &'a RefCell<RegistryManager>,
    pub(crate) manager: &'a mut AssetManager,
}

impl<'a> AssetContext<'a> {

    pub fn set_default<A: Asset>(&mut self, asset: UID, uid: UID) -> Result<()> {
        self.manager.set_default::<A>(asset, uid)
    }

    pub fn get<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<&'_ A> {
        self.manager.get::<A>(asset, uid)
    }

    pub fn get_or_default<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<&'_ A> {
        self.manager.get_or_default::<A>(asset, uid)
    }

    pub fn get_mut<A: Asset>(&'_ mut self, asset: UID, uid: UID) -> Result<&'_ mut A> {
        self.manager.get_mut::<A>(asset, uid)
    }

    pub fn entry<A: Asset>(&'_ self, asset: UID, uid: UID) -> Result<&'_ AssetEntry<A>> {
        self.manager.entry::<A>(asset, uid)
    }

    pub fn iter<A: Asset>(&'_ self, asset: UID) -> Result<impl Iterator<Item = (&UID, &'_ AssetEntry<A>)> + '_> {
        self.manager.iter::<A>(asset)
    }

    pub fn add_bundle(&mut self, name: &str) -> Result<()> {
        self.manager.add_bundle(name)
    }

    pub fn add<A: Asset>(&mut self, asset: UID, name: &str, bundle: UID, data: A) -> Result<()> {
        self.manager.add::<A>(&self.registry.borrow().assets, asset, name, bundle, data)
    }

    pub fn remove<A: Asset>(&mut self, asset: UID, uid: UID) -> Result<()> {
        self.manager.remove::<A>(asset, uid)
    }

    pub fn transfer<A: Asset>(&mut self, asset: UID, uid: UID, dst_bundle: UID) -> Result<()> {
        self.manager.transfer::<A>(asset, uid, dst_bundle)
    }
}