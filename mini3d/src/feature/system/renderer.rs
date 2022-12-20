use anyhow::Result;
use hecs::World;

use crate::{scene::SystemContext, feature::component::{camera::CameraComponent, model::ModelComponent, lifecycle::LifecycleComponent, ui::UIComponent}};

pub(crate) fn despawn_renderer_entities(ctx: &mut SystemContext, world: &mut World) -> Result<()> {

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
    for (_, (l, ui)) in world.query_mut::<(&LifecycleComponent, &mut UIComponent)>() {
        if !l.alive {
            if let Some(handle) = ui.handle { ctx.renderer.surface_canvases_removed.insert(handle); }
            if let Some(handle) = ui.ui.handle { ctx.renderer.canvases_removed.insert(handle); }
        }
    }

    Ok(())
}
