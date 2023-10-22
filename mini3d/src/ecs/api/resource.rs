use crate::{
    feature::core::resource,
    resource::{
        error::ResourceError,
        handle::{ResourceHandle, ToResourceHandle},
        hook::ResourceAddedHook,
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
        let mut hook = None;
        let handle = ctx.resource.add(ty, key, ctx.activity, data, &mut hook)?;
        if let Some(hook) = hook {
            match hook {
                ResourceAddedHook::Renderer(hook) => {
                    ctx.renderer
                        .on_resource_added_hook(hook, handle, ctx.resource);
                }
                ResourceAddedHook::Input(hook) => {
                    ctx.input.on_resource_added_hook(hook, handle, ctx.resource);
                }
            }
        }
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

    pub fn find_type(ctx: &Context, key: impl ToUID) -> Option<ResourceHandle> {
        ctx.resource.find_type(key)
    }
}
