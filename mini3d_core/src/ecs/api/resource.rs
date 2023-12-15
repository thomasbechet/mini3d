use crate::{
    resource::resource::{self, ResourceHookContext, ResourceType, ResourceTypeHandle},
    resource::{
        error::ResourceError,
        handle::{ResourceHandle, ToResourceHandle},
        ResourceInfo,
    },
    utils::uid::ToUID,
};

use super::Context;

pub struct Resource;

impl Resource {
    pub fn create<R: resource::Resource>(
        ctx: &mut Context,
        ty: ResourceTypeHandle,
        key: &str,
        data: R,
    ) -> Result<ResourceHandle, ResourceError> {
        let handle = ctx.resource.create(Some(key), ty, data)?;
        R::hook_added(
            handle,
            ResourceHookContext {
                input: ctx.input,
                renderer: ctx.renderer,
                resource: ctx.resource,
            },
        );
        Ok(handle)
    }

    pub fn create_any(ctx: &mut Context, ty: impl ToResourceHandle, key: Option<&str>) {
        todo!()
    }

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<ResourceHandle> {
        ctx.resource.find(key)
    }

    pub fn iter<'a>(ctx: &'a Context) -> impl Iterator<Item = ResourceHandle> + 'a {
        ctx.resource.iter()
    }

    pub fn info<'a>(
        ctx: &'a Context,
        handle: impl ToResourceHandle,
    ) -> Result<ResourceInfo<'a>, ResourceError> {
        ctx.resource.info(handle)
    }
}

impl ResourceType {
    pub fn create(
        ctx: &mut Context,
        key: &str,
        ty: ResourceType,
    ) -> Result<ResourceTypeHandle, ResourceError> {
        ctx.resource.create_resource_type(Some(key), ty)
    }

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<ResourceTypeHandle> {
        ctx.resource.find_type(key)
    }
}
