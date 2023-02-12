use core::cell::RefCell;
use anyhow::Result;

use crate::{asset::{AssetManager, AssetEntry}, uid::UID, registry::RegistryManager};

pub struct AssetContext<'a> {
    registry: &'a RefCell<RegistryManager>,
    asset: &'a RefCell<AssetManager>,   
}

impl<'a> AssetContext<'a> {

    pub(crate) fn new(registry: &'a RefCell<RegistryManager>, asset: &'a RefCell<AssetManager>) -> Self {
        Self { registry, asset }
    }

    pub fn set_default<A: 'static>(&mut self, uid: UID) -> Result<()> {
        self.asset.borrow_mut().set_default::<A>(uid)
    }

    pub fn get<A: 'static>(&'_ self, uid: UID) -> Result<&'_ A> {
        self.asset.borrow().get::<A>(self.registry.borrow(), uid)
    }

    pub fn get_or_default<A: 'static>(&'_ self, uid: UID) -> Result<&'_ A> {
        self.asset.borrow().get_or_default::<A>(self.registry.borrow(), uid)
    }

    pub fn get_mut<A: 'static>(&'_ mut self, uid: UID) -> Result<&'_ mut A> {
        self.asset.borrow_mut().get_mut::<A>(self.registry.borrow(), uid)
    }

    pub fn entry<A: 'static>(&'_ self, uid: UID) -> Result<&'_ AssetEntry<A>> {
        self.asset.borrow().entry::<A>(self.registry.borrow(), uid)
    }

    pub fn iter<A: 'static>(&'_ self) -> Result<impl Iterator<Item = (&UID, &'_ AssetEntry<A>)>> {
        self.asset.borrow().iter::<A>(self.registry.borrow())
    }

    pub fn add_bundle(&mut self, name: &str) -> Result<()> {
        self.asset.borrow_mut().add_bundle(name)
    }

    pub fn add<A: 'static>(&mut self, name: &str, bundle: UID, data: A) -> Result<()> {
        self.asset.borrow_mut().add::<A>(self.registry.borrow(), name, bundle, data)
    }

    pub fn remove<A: 'static>(&mut self, uid: UID) -> Result<()> {
        self.asset.borrow_mut().remove::<A>(self.registry.borrow(), uid)
    }

    pub fn transfer<A: 'static>(&mut self, uid: UID, dst_bundle: UID) -> Result<()> {
        self.asset.borrow_mut().transfer::<A>(self.registry.borrow(), uid, dst_bundle)
    }
}