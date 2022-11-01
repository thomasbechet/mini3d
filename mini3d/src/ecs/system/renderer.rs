use anyhow::Result;
use hecs::World;
use slotmap::Key;

use crate::{backend::renderer::RendererBackend, ecs::component::{transform::TransformComponent, camera::CameraComponent, lifecycle::LifecycleComponent, model::ModelComponent}, asset::AssetManager};

pub fn system_renderer_check_lifecycle(
    world: &mut World,
    renderer: &mut dyn RendererBackend,
    asset: &AssetManager,
) -> Result<()> {
    // Camera
    for (_, (l, c)) in world.query_mut::<(&LifecycleComponent, &mut CameraComponent)>() {
        if l.alive && c.renderer_id.is_null() {
            c.submit(renderer)?;
        } else if !l.alive && !c.renderer_id.is_null() {
            c.release(renderer)?;
        }
    }

    // Model
    for (_, (l, m)) in world.query_mut::<(&LifecycleComponent, &mut ModelComponent)>() {
        if l.alive && m.renderer_id.is_null() {
            m.submit(asset, renderer)?;
        } else if !l.alive && !m.renderer_id.is_null() {
            m.release(renderer)?;
        }
    }

    Ok(())
}

pub fn system_renderer_transfer_transforms(
    world: &mut World,
    renderer: &mut dyn RendererBackend,
) -> Result<()> {
    for (_, (t, m)) in world.query_mut::<(&TransformComponent, &ModelComponent)>() {
        renderer.update_model_transform(m.renderer_id, t.matrix())?;
    }
    Ok(())
}

pub fn system_renderer_update_camera(
    world: &mut World,
    renderer: &mut dyn RendererBackend,
) -> Result<()> {
    for (_, (t, c)) in world.query_mut::<(&TransformComponent, &CameraComponent)>() {
        renderer.update_camera(c.renderer_id, t.translation, t.forward(), t.up(), c.fov)?;
    }
    Ok(())
}