use anyhow::Result;
use hecs::{CommandBuffer, World};

use crate::{scene::SystemContext, feature::component::lifecycle::LifecycleComponent};

pub fn run(_ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    let mut cb = CommandBuffer::default();
    for (e, l) in world.query_mut::<&LifecycleComponent>() {
        if !l.alive {
            cb.despawn(e);
        }
    }
    cb.run_on(world);
    Ok(())
}