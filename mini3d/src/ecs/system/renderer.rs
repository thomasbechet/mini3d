use anyhow::Result;
use hecs::World;
use slotmap::Key;

use crate::{backend::renderer::RendererModelDescriptor, ecs::component::{transform::TransformComponent, camera::CameraComponent, lifecycle::LifecycleComponent, model::ModelComponent}};

use super::{System, SystemContext};

pub struct RendererCheckLifecycleSystem;

impl System for RendererCheckLifecycleSystem {
    fn run(&self, ctx: &mut SystemContext, world: &mut World) -> Result<()> {

        // Camera
        for (_, (l, c)) in world.query_mut::<(&LifecycleComponent, &mut CameraComponent)>() {
            if l.alive && c.id.is_null() {
                c.id = ctx.renderer.add_camera()?;
            } else if !l.alive && !c.id.is_null() {
                ctx.renderer.remove_camera(c.id)?;
            }
        }

        // Model
        for (_, (l, m)) in world.query_mut::<(&LifecycleComponent, &mut ModelComponent)>() {
            if l.alive && m.id.is_null() {
                m.id = ctx.renderer.add_model(&RendererModelDescriptor::FromAsset(m.uid), ctx.asset)?;
            } else if !l.alive && !m.id.is_null() {
                ctx.renderer.remove_model(m.id)?;
            }
        }

        Ok(())
    }
}

pub struct RendererTransferTransformsSystem;

impl System for RendererTransferTransformsSystem {
    fn run(&self, ctx: &mut SystemContext, world: &mut World) -> Result<()> {
        for (_, (t, c)) in world.query_mut::<(&TransformComponent, &CameraComponent)>() {
            ctx.renderer.update_camera(c.id, t.translation, t.forward(), t.up(), c.fov)?;
        }
        Ok(())
    }
}

pub struct RendererUpdateCameraSystem;

impl System for RendererUpdateCameraSystem {
    fn run(&self, ctx: &mut SystemContext, world: &mut World) -> Result<()> {
        for (_, (t, m)) in world.query_mut::<(&TransformComponent, &ModelComponent)>() {
            ctx.renderer.update_model_transform(m.id, t.matrix())?;
        }
        Ok(())
    }
}