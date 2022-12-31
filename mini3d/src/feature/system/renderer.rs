use anyhow::Result;
use hecs::World;

use crate::{scene::SystemContext, feature::component::{camera::CameraComponent, model::ModelComponent, lifecycle::LifecycleComponent, viewport::ViewportComponent, canvas::CanvasComponent}};

pub(crate) fn despawn_renderer_entities(ctx: &mut SystemContext, world: &mut World) -> Result<()> {

    for (_, (l, v)) in world.query_mut::<(&LifecycleComponent, &mut ViewportComponent)>() {
        if !l.alive {
            if let Some(handle) = v.handle { ctx.renderer.viewports_removed.insert(handle); }
        }
    }
    for (_, (l, c)) in world.query_mut::<(&LifecycleComponent, &mut CameraComponent)>() {
        if !l.alive { 
            if let Some(handle) = c.handle { ctx.renderer.scene_cameras_removed.insert(handle); }
        }
    }
    for (_, (l, m)) in world.query_mut::<(&LifecycleComponent, &mut ModelComponent)>() {
        if !l.alive { 
            if let Some(handle) = m.handle { ctx.renderer.scene_models_removed.insert(handle); }
        }
    }
    for (_, (l, c)) in world.query_mut::<(&LifecycleComponent, &mut CanvasComponent)>() {
        if !l.alive {
            if let Some(handle) = c.handle { ctx.renderer.scene_canvases_removed.insert(handle); }
        }
    }

    Ok(())
}
