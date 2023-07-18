use crate::{
    asset::{container::AssetEntry, error::AssetError, AssetManager},
    registry::component::{Component, ComponentId, ComponentRegistry},
    utils::uid::UID,
};

pub struct ExclusiveAssetContext<'a> {
    pub(crate) registry: &'a ComponentRegistry,
    pub(crate) manager: &'a mut AssetManager,
}

impl<'a> ExclusiveAssetContext<'a> {
    pub fn set_default(&mut self, asset: ComponentId, uid: UID) -> Result<(), AssetError> {
        self.manager.set_default(asset, uid)
    }

    pub fn get<C: Component>(
        &'_ self,
        asset: ComponentId,
        uid: UID,
    ) -> Result<Option<&'_ C>, AssetError> {
        self.manager.get::<C>(asset, uid)
    }

    pub fn get_or_default<C: Component>(
        &'_ self,
        asset: ComponentId,
        uid: UID,
    ) -> Result<Option<&'_ C>, AssetError> {
        self.manager.get_or_default::<C>(asset, uid)
    }

    pub fn entry<C: Component>(
        &'_ self,
        asset: ComponentId,
        uid: UID,
    ) -> Result<Option<&'_ AssetEntry<C>>, AssetError> {
        self.manager.entry::<C>(asset, uid)
    }

    pub fn iter<C: Component>(
        &self,
        asset: ComponentId,
    ) -> Result<Option<impl Iterator<Item = &AssetEntry<C>>>, AssetError> {
        self.manager.iter::<C>(asset)
    }

    pub fn add_bundle(&mut self, name: &str) -> Result<UID, AssetError> {
        self.manager.add_bundle(name)
    }

    pub fn add<C: Component>(
        &mut self,
        asset: ComponentId,
        name: &str,
        bundle: UID,
        data: C,
    ) -> Result<(), AssetError> {
        self.manager
            .add::<C>(&self.registry, asset, name, bundle, data)
    }

    pub fn remove<C: Component>(&mut self, asset: ComponentId, uid: UID) -> Result<(), AssetError> {
        self.manager.remove::<C>(asset, uid)
    }

    pub fn transfer<C: Component>(
        &mut self,
        asset: ComponentId,
        uid: UID,
        dst_bundle: UID,
    ) -> Result<(), AssetError> {
        self.manager.transfer::<C>(asset, uid, dst_bundle)
    }
}

pub struct ParallelAssetContext<'a> {
    pub(crate) manager: &'a AssetManager,
}

impl<'a> ParallelAssetContext<'a> {
    pub fn get<C: Component>(
        &'_ self,
        asset: ComponentId,
        uid: UID,
    ) -> Result<Option<&'_ C>, AssetError> {
        self.manager.get::<C>(asset, uid)
    }

    pub fn get_or_default<C: Component>(
        &'_ self,
        asset: ComponentId,
        uid: UID,
    ) -> Result<Option<&'_ C>, AssetError> {
        self.manager.get_or_default::<C>(asset, uid)
    }

    pub fn entry<C: Component>(
        &'_ self,
        asset: ComponentId,
        uid: UID,
    ) -> Result<Option<&'_ AssetEntry<C>>, AssetError> {
        self.manager.entry::<C>(asset, uid)
    }

    pub fn iter<C: Component>(
        &self,
        asset: ComponentId,
    ) -> Result<Option<impl Iterator<Item = &AssetEntry<C>>>, AssetError> {
        self.manager.iter::<C>(asset)
    }
}
