use crate::{
    asset::{error::AssetError, handle::AssetHandle, AssetInfo},
    registry::asset::{AssetReferenceTrait, AssetTypeTrait},
    utils::uid::ToUID,
};

use super::context::Context;

pub struct Asset;

impl Asset {
    pub fn persist<T: AssetTypeTrait>(
        ctx: &mut Context,
        ty: T,
        key: &str,
        data: T::Data,
    ) -> Result<AssetHandle, AssetError> {
        ctx.asset.persist(ty, key, data)
    }

    pub fn remove(ctx: &mut Context, handle: AssetHandle) -> Result<(), AssetError> {
        ctx.asset.remove(handle)
    }

    pub fn load<'a, T: AssetTypeTrait>(
        ctx: &mut Context,
        handle: AssetHandle,
    ) -> Result<T::Ref<'a>, AssetError> {
        // ctx.asset.load(handle)
        todo!()
    }

    pub fn read<'a, T: AssetReferenceTrait>(
        ctx: &'a Context,
        handle: AssetHandle,
    ) -> Result<<T::AssetType as AssetTypeTrait>::Ref<'a>, AssetError> {
        ctx.asset.read::<T>(handle)
    }

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<AssetHandle> {
        ctx.asset.find(key)
    }

    pub fn info<'a>(ctx: &'a Context, handle: AssetHandle) -> Result<AssetInfo<'a>, AssetError> {
        ctx.asset.info(handle)
    }
}
