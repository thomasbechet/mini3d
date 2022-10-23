use hecs::{World, CommandBuffer};

use crate::ecs::component::lifecycle::LifecycleComponent;

pub fn system_despawn_entities(
    world: &mut World,
) {
    let mut cb = CommandBuffer::default();
    for (e, l) in world.query_mut::<&LifecycleComponent>() {
        if !l.alive {
            cb.despawn(e);
        }
    }
    cb.run_on(world);
}