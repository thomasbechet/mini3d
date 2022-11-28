use anyhow::Result;
use hecs::World;

use crate::{scene::SystemContext, feature::component::{lifecycle::LifecycleComponent, camera::CameraComponent, model::ModelComponent, transform::TransformComponent}, backend::renderer::RendererModelDescriptor};

pub fn check_lifecycle(ctx: &mut SystemContext, world: &mut World) -> Result<()> {

    // Camera
    for (_, (l, c)) in world.query_mut::<(&LifecycleComponent, &mut CameraComponent)>() {
        if l.alive && c.handle.is_null() {
            c.handle = ctx.renderer.add_camera()?;
        } else if !l.alive && !c.handle.is_null() {
            ctx.renderer.remove_camera(c.handle)?;
        }
    }

    // Model
    for (_, (l, m)) in world.query_mut::<(&LifecycleComponent, &mut ModelComponent)>() {
        if l.alive && m.handle.is_null() {
            m.handle = ctx.renderer.add_model(&RendererModelDescriptor::FromAsset(m.model), ctx.asset)?;
        } else if !l.alive && !m.handle.is_null() {
            ctx.renderer.remove_model(m.handle)?;
        }
    }

    Ok(())
}

pub fn transfer_transforms(ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    for (_, (t, m)) in world.query_mut::<(&TransformComponent, &ModelComponent)>() {
        ctx.renderer.update_model_transform(m.handle, t.matrix())?;
    }
    Ok(())
}

pub fn update_camera(ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    for (_, (t, c)) in world.query_mut::<(&TransformComponent, &CameraComponent)>() {
        ctx.renderer.update_camera(c.handle, t.translation, t.forward(), t.up(), c.fov)?;
    }
    Ok(())
}