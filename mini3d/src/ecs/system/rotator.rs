use glam::Mat4;
use hecs::World;

use crate::ecs::component::{transform::TransformComponent, rotator::RotatorComponent};

pub fn system_rotator(
    world: &mut World,
) {
    for (_, (t, _)) in world.query_mut::<(&mut TransformComponent, &RotatorComponent)>() {
        t.matrix *= Mat4::from_rotation_y(0.005);
    }
}