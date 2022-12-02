use anyhow::Result;
use hecs::World;

use crate::{scene::SystemContext, feature::component::{camera::CameraComponent, model::ModelComponent, lifecycle::LifecycleComponent}};

pub(crate) fn despawn_renderer_entities(ctx: &mut SystemContext, world: &mut World) -> Result<()> {

    for (_, (l, c)) in world.query_mut::<(&LifecycleComponent, &mut CameraComponent)>() {
        if !l.alive { 
            if let Some(handle) = c.handle { ctx.renderer.cameras_removed.insert(handle); }
        }
    }
    for (_, (l, m)) in world.query_mut::<(&LifecycleComponent, &mut ModelComponent)>() {
        if !l.alive { 
            if let Some(handle) = m.handle { ctx.renderer.models_removed.insert(handle); }
        }
    }

    Ok(())
}
