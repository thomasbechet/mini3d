use crate::{
    feature::core::resource::{self, ResourceHookContext, ResourceType},
    resource::{
        error::ResourceError,
        handle::{ResourceHandle, ToResourceHandle, ResourceTypeHandle},
    },
    utils::uid::ToUID,
};

use super::Context;

pub struct Resource;

impl Resource {
    pub fn add<R: resource::Resource>(
        ctx: &mut Context,
        ty: impl ToResourceHandle,
        key: Option<&str>,
        data: R,
    ) -> Result<ResourceHandle, ResourceError> {
        let handle = ctx.resource.add(ty, key, ctx.activity, data)?;
        R::hook_added(
            handle,
            ResourceHookContext {
                input: &mut ctx.input,
                renderer: &mut ctx.renderer,
                resource: &mut ctx.resource,
            },
        );
        Ok(handle)
    }

    pub fn add_any(ctx: &mut Context, ty: impl ToResourceHandle, key: Option<&str>) {
        todo!()
    }

    pub fn clone(ctx: &mut Context, handle: impl ToResourceHandle) -> ResourceHandle {}

    pub fn find(
        ctx: &Context,
        ty: impl ToResourceHandle,
        key: impl ToUID,
    ) -> Option<ResourceHandle> {
        ctx.resource.find(ty, key)
    }

    pub fn find_type(ctx: &Context, key: impl ToUID) -> Option<ResourceTypeHandle> {
        ctx.resource.find_type(key)
    }

    pub fn add_type(ctx: &mut Context, ty: impl ToResourceHandle) -> ResourceTypeHandle {
        ResourceType {

        }
        ctx.resource.add(ctx.resource., key, owner, data)
    }
}
