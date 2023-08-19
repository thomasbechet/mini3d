use crate::{
    asset::{
        error::AssetError,
        handle::{AssetBundleId, AssetHandle},
        AssetManager, AssetSource,
    },
    registry::component::{ComponentHandle, ComponentRegistry},
};

pub struct ExclusiveAssetContext<'a> {
    pub(crate) registry: &'a ComponentRegistry,
    pub(crate) manager: &'a mut AssetManager,
}

impl<'a> ExclusiveAssetContext<'a> {
    pub fn add_bundle(&mut self, name: &str) -> Result<AssetBundleId, AssetError> {
        self.manager.add_bundle(name)
    }

    pub fn add<C: ComponentHandle>(
        &mut self,
        handle: C,
        name: &str,
        bundle: AssetBundleId,
        data: <C::AssetHandle as AssetHandle>::Data,
    ) -> Result<C::AssetHandle, AssetError> {
        self.manager.add(
            handle,
            name,
            bundle,
            AssetSource::Persistent,
            data,
            self.registry,
        )
    }

    pub fn remove<H: AssetHandle>(&mut self, handle: H) -> Result<(), AssetError> {
        self.manager.remove(handle)
    }

    pub fn read<H: AssetHandle>(&self, handle: H) -> Result<H::AssetRef<'_>, AssetError> {
        self.manager.read(handle)
    }

    pub fn write<H: AssetHandle>(
        &self,
        handle: H,
        asset: H::AssetRef<'_>,
    ) -> Result<(), AssetError> {
        self.manager.write(handle, asset)
    }
}

pub struct ParallelAssetContext<'a> {
    pub(crate) manager: &'a mut AssetManager,
}

impl<'a> ParallelAssetContext<'a> {
    pub fn read<H: AssetHandle>(&mut self, handle: H) -> Result<H::AssetRef<'_>, AssetError> {
        self.manager.read(handle)
    }
}
