use anyhow::Result;
use hecs::World;
use slotmap::Key;

use crate::{backend::renderer::{RendererBackend, RendererModelDescriptor}, ecs::component::{transform::TransformComponent, camera::CameraComponent, lifecycle::LifecycleComponent, model::ModelComponent}, asset::AssetManager};

pub fn system_renderer_check_lifecycle(
    world: &mut World,
    renderer: &mut dyn RendererBackend,
    asset: &AssetManager,
) -> Result<()> {
    // Camera
    for (_, (l, c)) in world.query_mut::<(&LifecycleComponent, &mut CameraComponent)>() {
        if l.alive && c.id.is_null() {
            c.id = renderer.add_camera()?;
        } else if !l.alive && !c.id.is_null() {
            renderer.remove_camera(c.id)?;
        }
    }

    // Model
    for (_, (l, m)) in world.query_mut::<(&LifecycleComponent, &mut ModelComponent)>() {
        if l.alive && m.id.is_null() {
            m.id = renderer.add_model(&RendererModelDescriptor::FromAsset(m.uid), asset)?;
        } else if !l.alive && !m.id.is_null() {
            renderer.remove_model(m.id)?;
        }
    }

    Ok(())
}

pub fn system_renderer_transfer_transforms(
    world: &mut World,
    renderer: &mut dyn RendererBackend,
) -> Result<()> {
    for (_, (t, m)) in world.query_mut::<(&TransformComponent, &ModelComponent)>() {
        renderer.update_model_transform(m.id, t.matrix())?;
    }
    Ok(())
}

pub fn system_renderer_update_camera(
    world: &mut World,
    renderer: &mut dyn RendererBackend,
) -> Result<()> {
    for (_, (t, c)) in world.query_mut::<(&TransformComponent, &CameraComponent)>() {
        renderer.update_camera(c.id, t.translation, t.forward(), t.up(), c.fov)?;
    }
    Ok(())
}