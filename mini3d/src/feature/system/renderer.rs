use crate::{
    context::SystemContext,
    ecs::system::SystemResult,
    feature::component::{
        common::lifecycle::Lifecycle,
        renderer::{camera::Camera, static_mesh::StaticMesh},
        ui::{canvas::Canvas, viewport::Viewport},
    },
    registry::component::Component,
};

pub(crate) fn despawn_renderer_entities(ctx: &mut SystemContext) -> SystemResult {
    let scene = ctx.scene.active();
    let lifecycles = scene.static_view::<Lifecycle>(Lifecycle::UID)?;
    let viewports = scene.static_view::<Viewport>(Viewport::UID)?;
    let cameras = scene.static_view::<Camera>(Camera::UID)?;
    let models = scene.static_view::<StaticMesh>(StaticMesh::UID)?;
    let canvases = scene.static_view::<Canvas>(Canvas::UID)?;

    for e in &scene.query(&[Lifecycle::UID, Viewport::UID]) {
        if !lifecycles[e].alive {
            if let Some(handle) = viewports[e].handle {
                ctx.renderer.manager.viewports_removed.insert(handle);
            }
        }
    }

    for e in &scene.query(&[Lifecycle::UID, Camera::UID]) {
        if !lifecycles[e].alive {
            if let Some(handle) = cameras[e].handle {
                ctx.renderer.manager.scene_cameras_removed.insert(handle);
            }
        }
    }

    for e in &scene.query(&[Lifecycle::UID, StaticMesh::UID]) {
        if !lifecycles[e].alive {
            if let Some(handle) = models[e].handle {
                ctx.renderer.manager.scene_models_removed.insert(handle);
            }
        }
    }

    for e in &scene.query(&[Lifecycle::UID, Canvas::UID]) {
        if !lifecycles[e].alive {
            if let Some(handle) = canvases[e].handle {
                ctx.renderer.manager.scene_canvases_removed.insert(handle);
            }
        }
    }

    Ok(())
}
