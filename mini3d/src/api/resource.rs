use crate::{
    feature::core::resource::{self, ResourceHookContext, ResourceType, ResourceTypeHandle},
    resource::{
        error::ResourceError,
        handle::{ResourceHandle, ToResourceHandle},
    },
    utils::uid::ToUID,
};

use super::Context;

pub struct Resource;

impl Resource {
    pub fn add<R: resource::Resource>(
        ctx: &mut Context,
        ty: ResourceTypeHandle,
        key: Option<&str>,
        data: R,
    ) -> Result<ResourceHandle, ResourceError> {
        let handle = ctx.resource.add(data, ty, ctx.activity.active, key)?;
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

    pub fn find(ctx: &Context, key: impl ToUID) -> Option<ResourceHandle> {
        ctx.resource.find(key)
    }

    pub fn find_type(ctx: &Context, key: impl ToUID) -> Option<ResourceTypeHandle> {
        ctx.resource.find_type(key)
    }

    pub fn define_type(
        ctx: &mut Context,
        name: &str,
        ty: ResourceType,
    ) -> Result<ResourceTypeHandle, ResourceError> {
        ctx.resource.define_type(name, ty, ctx.activity.active)
    }
}
