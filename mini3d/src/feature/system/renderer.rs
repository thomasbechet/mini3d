use crate::{
    context::SystemContext,
    ecs::system::SystemResult,
    feature::component::{
        camera::Camera, canvas::Canvas, lifecycle::Lifecycle, static_mesh::StaticMesh,
        viewport::Viewport,
    },
    registry::component::Component,
};

pub(crate) fn despawn_renderer_entities(ctx: &mut SystemContext) -> SystemResult {
    let world = ctx.world.active();
    let lifecycles = world.static_view::<Lifecycle>(Lifecycle::UID)?;
    let viewports = world.static_view::<Viewport>(Viewport::UID)?;
    let cameras = world.static_view::<Camera>(Camera::UID)?;
    let models = world.static_view::<StaticMesh>(StaticMesh::UID)?;
    let canvases = world.static_view::<Canvas>(Canvas::UID)?;

    for e in &world.query(&[Lifecycle::UID, Viewport::UID]) {
        if !lifecycles[e].alive {
            if let Some(handle) = viewports[e].handle {
                ctx.renderer.manager.viewports_removed.insert(handle);
            }
        }
    }

    for e in &world.query(&[Lifecycle::UID, Camera::UID]) {
        if !lifecycles[e].alive {
            if let Some(handle) = cameras[e].handle {
                ctx.renderer.manager.scene_cameras_removed.insert(handle);
            }
        }
    }

    for e in &world.query(&[Lifecycle::UID, StaticMesh::UID]) {
        if !lifecycles[e].alive {
            if let Some(handle) = models[e].handle {
                ctx.renderer.manager.scene_models_removed.insert(handle);
            }
        }
    }

    for e in &world.query(&[Lifecycle::UID, Canvas::UID]) {
        if !lifecycles[e].alive {
            if let Some(handle) = canvases[e].handle {
                ctx.renderer.manager.scene_canvases_removed.insert(handle);
            }
        }
    }

    Ok(())
}
