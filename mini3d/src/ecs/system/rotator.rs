use glam::{Quat, Vec3};
use hecs::World;

use crate::ecs::component::{transform::TransformComponent, rotator::RotatorComponent};

pub fn system_rotator(
    world: &mut World,
    delta_time: f32,
) {
    for (_, (transform, rotator)) in world.query_mut::<(&mut TransformComponent, &RotatorComponent)>() {
        transform.rotation *= Quat::from_axis_angle(Vec3::Y, delta_time * f32::to_radians(rotator.speed));
    }
}