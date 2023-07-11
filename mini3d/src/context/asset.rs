use std::cell::RefCell;

use crate::{
    asset::{AssetEntry, AssetError, AssetManager},
    registry::{component::Component, RegistryManager},
    uid::UID,
};

pub struct AssetContext<'a> {
    pub(crate) registry: &'a RefCell<RegistryManager>,
    pub(crate) manager: &'a mut AssetManager,
}

impl<'a> AssetContext<'a> {
    pub fn set_default(&mut self, asset: UID, uid: UID) -> Result<(), AssetError> {
        self.manager.set_default(asset, uid)
    }

    pub fn get<C: Component>(&'_ self, asset: UID, uid: UID) -> Result<Option<&'_ C>, AssetError> {
        self.manager.get::<C>(asset, uid)
    }

    pub fn get_or_default<C: Component>(
        &'_ self,
        asset: UID,
        uid: UID,
    ) -> Result<Option<&'_ C>, AssetError> {
        self.manager.get_or_default::<C>(asset, uid)
    }

    pub fn entry<C: Component>(
        &'_ self,
        asset: UID,
        uid: UID,
    ) -> Result<Option<&'_ AssetEntry<C>>, AssetError> {
        self.manager.entry::<C>(asset, uid)
    }

    pub fn iter<C: Component>(
        &self,
        asset: UID,
    ) -> Result<Option<impl Iterator<Item = &AssetEntry<C>>>, AssetError> {
        self.manager.iter::<C>(asset)
    }

    pub fn add_bundle(&mut self, name: &str) -> Result<UID, AssetError> {
        self.manager.add_bundle(name)
    }

    pub fn add<C: Component>(
        &mut self,
        asset: UID,
        name: &str,
        bundle: UID,
        data: C,
    ) -> Result<(), AssetError> {
        self.manager.add::<C>(
            &self.registry.borrow().components,
            asset,
            name,
            bundle,
            data,
        )
    }

    pub fn remove<C: Component>(&mut self, asset: UID, uid: UID) -> Result<(), AssetError> {
        self.manager.remove::<C>(asset, uid)
    }

    pub fn transfer<C: Component>(
        &mut self,
        asset: UID,
        uid: UID,
        dst_bundle: UID,
    ) -> Result<(), AssetError> {
        self.manager.transfer::<C>(asset, uid, dst_bundle)
    }
}
