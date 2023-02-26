use anyhow::Result;

use crate::{feature::component::{camera::Camera, model::Model, lifecycle::Lifecycle, viewport::Viewport, canvas::Canvas}, context::SystemContext};

pub(crate) fn despawn_renderer_entities(ctx: &mut SystemContext) -> Result<()> {

    let world = ctx.world.active();
    let lifecycles = world.view::<Lifecycle>(Lifecycle::UID)?;
    let viewports = world.view::<Viewport>(Viewport::UID)?;
    let cameras = world.view::<Camera>(Camera::UID)?;
    let models = world.view::<Model>(Model::UID)?;
    let canvases = world.view::<Canvas>(Canvas::UID)?;

    for e in &world.query(&[Lifecycle::UID, Viewport::UID]) {
        if !lifecycles[e].alive {
            if let Some(handle) = viewports[e].handle { ctx.renderer.manager.viewports_removed.insert(handle); }
        }
    }

    for e in &world.query(&[Lifecycle::UID, Camera::UID]) {
        if !lifecycles[e].alive { 
            if let Some(handle) = cameras[e].handle { ctx.renderer.manager.scene_cameras_removed.insert(handle); }
        }
    }

    for e in &world.query(&[Lifecycle::UID, Model::UID]) {
        if !lifecycles[e].alive { 
            if let Some(handle) = models[e].handle { ctx.renderer.manager.scene_models_removed.insert(handle); }
        }
    }

    for e in &world.query(&[Lifecycle::UID, Canvas::UID]) {
        if !lifecycles[e].alive { 
            if let Some(handle) = canvases[e].handle { ctx.renderer.manager.scene_canvases_removed.insert(handle); }
        }
    }

    Ok(())
}
