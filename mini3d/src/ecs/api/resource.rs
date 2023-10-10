use crate::{
    resource::{error::ResourceError, handle::ResourceHandle, ResourceInfo},
    utils::uid::ToUID,
};

use super::context::Context;

pub struct Resource;

impl Resource {
    pub fn persist<T: ResourceTypeTrait>(
        ctx: &mut Context,
        ty: T,
        key: &str,
        data: T::Data,
    ) -> Result<ResourceHandle, ResourceError> {
        ctx.resource.persist(ty, key, data)
    }

    pub fn remove(ctx: &mut Context, handle: ResourceHandle) -> Result<(), ResourceError> {
        ctx.resource.remove(handle)
    }

    pub fn load<'a, T: ResourceTypeTrait>(
        ctx: &mut Context,
        handle: ResourceHandle,
    ) -> Result<T::Ref<'a>, ResourceError> {
        // ctx.resource.load(handle)
        todo!()
    }

    pub fn read<'a, T: ResourceReferenceTrait>(
        ctx: &'a Context,
        handle: ResourceHandle,
    ) -> Result<<T::AssetType as ResourceTypeTrait>::Ref<'a>, ResourceError> {
        ctx.resource.read::<T>(handle)
    }

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<ResourceHandle> {
        ctx.resource.find(key)
    }

    pub fn info<'a>(
        ctx: &'a Context,
        handle: ResourceHandle,
    ) -> Result<ResourceInfo<'a>, ResourceError> {
        ctx.resource.info(handle)
    }
}
