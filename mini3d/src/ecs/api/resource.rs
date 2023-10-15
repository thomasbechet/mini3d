use crate::{
    activity::ActivityId,
    feature::core::resource_type,
    resource::{
        error::ResourceError,
        handle::{ResourceHandle, ToResourceHandle},
        hook::ResourceAddedHook,
    },
};

use super::context::Context;

pub struct Resource;

impl Resource {
    pub fn add<R: resource_type::Resource>(
        ctx: &mut Context,
        ty: impl ToResourceHandle,
        key: &str,
        owner: ActivityId,
        data: R,
    ) -> Result<ResourceHandle, ResourceError> {
        let mut hook = None;
        let handle = ctx.resource.add(ty, key, owner, data, &mut hook)?;
        if let Some(hook) = hook {
            match hook {
                ResourceAddedHook::Renderer(hook) => {
                    ctx.renderer
                        .on_resource_added_hook(hook, handle, ctx.resource);
                }
            }
        }
        Ok(handle)
    }

    pub fn add_any(ctx: &mut Context, ty: impl ToResourceHandle, key: &str, owner: ActivityId) {
        todo!()
    }
}
