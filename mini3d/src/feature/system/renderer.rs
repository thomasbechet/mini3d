use anyhow::Result;
use hecs::World;

use crate::{scene::SystemContext, feature::component::{camera::CameraComponent, model::ModelComponent, lifecycle::LifecycleComponent}, renderer::RendererModelDescriptor};

pub(crate) fn synchronize_renderer(ctx: &mut SystemContext, world: &mut World) -> Result<()> {

    // Check camera lifecycle
    for (_, (l, c)) in world.query_mut::<(&LifecycleComponent, &mut CameraComponent)>() {
        if l.alive && c.handle.is_none() {
            c.handle = Some(ctx.renderer.add_camera()?);
        } else if !l.alive && c.handle.is_some() {
            ctx.renderer.remove_camera(c.handle.unwrap())?;
        }
    }

    // Check model lifecycle
    for (_, (l, m)) in world.query_mut::<(&LifecycleComponent, &mut ModelComponent)>() {
        if l.alive && m.handle.is_none() {
            m.handle = Some(ctx.renderer.add_model(RendererModelDescriptor::FromAsset(m.model), ctx.asset)?);
        } else if !l.alive && m.handle.is_some() {
            ctx.renderer.remove_model(m.handle.unwrap())?;
        }
    }

    Ok(())
}
