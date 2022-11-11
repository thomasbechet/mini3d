use anyhow::Result;
use hecs::{CommandBuffer, World};

use crate::ecs::component::lifecycle::LifecycleComponent;

use super::{System, SystemContext};

pub struct DespawnEntitiesSystem;

impl System for DespawnEntitiesSystem {
    fn run(&self, _ctx: &mut SystemContext, world: &mut World) -> Result<()> {
        let mut cb = CommandBuffer::default();
        for (e, l) in world.query_mut::<&LifecycleComponent>() {
            if !l.alive {
                cb.despawn(e);
            }
        }
        cb.run_on(world);
        Ok(())
    }
}