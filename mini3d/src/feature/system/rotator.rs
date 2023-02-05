use anyhow::Result;
use glam::{Quat, Vec3};

use crate::{feature::component::{transform::Transform, rotator::Rotator}, scene::{context::SystemContext, world::World}};

pub fn run(ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    for (_, (transform, rotator)) in world.query_mut::<(&mut Transform, &Rotator)>() {
        transform.rotation *= Quat::from_axis_angle(Vec3::Y, ctx.delta_time as f32 * f32::to_radians(rotator.speed));
    }
    Ok(())
}