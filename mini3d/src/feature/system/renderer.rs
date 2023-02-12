use anyhow::Result;

use crate::{feature::component::{camera::Camera, model::Model, lifecycle::Lifecycle, viewport::Viewport, canvas::Canvas}, scene::{context::SystemContext, world::World}};

pub(crate) fn despawn_renderer_entities(ctx: &mut SystemContext) -> Result<()> {

    for (_, (l, v)) in world.query_mut::<(&Lifecycle, &mut Viewport)>() {
        if !l.alive {
            if let Some(handle) = v.handle { ctx.renderer.viewports_removed.insert(handle); }
        }
    }
    for (_, (l, c)) in world.query_mut::<(&Lifecycle, &mut Camera)>() {
        if !l.alive { 
            if let Some(handle) = c.handle { ctx.renderer.scene_cameras_removed.insert(handle); }
        }
    }
    for (_, (l, m)) in world.query_mut::<(&Lifecycle, &mut Model)>() {
        if !l.alive { 
            if let Some(handle) = m.handle { ctx.renderer.scene_models_removed.insert(handle); }
        }
    }
    for (_, (l, c)) in world.query_mut::<(&Lifecycle, &mut Canvas)>() {
        if !l.alive {
            if let Some(handle) = c.handle { ctx.renderer.scene_canvases_removed.insert(handle); }
        }
    }

    Ok(())
}
