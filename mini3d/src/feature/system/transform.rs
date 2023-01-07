use anyhow::Result;
use hecs::World;

use crate::{scene::SystemContext, feature::component::transform::TransformComponent};

pub fn run(ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    // for (_, (transform, rotator)) in world.query_mut::<(&mut TransformComponent, &RotatorComponent)>() {
    //     transform.rotation *= Quat::from_axis_angle(Vec3::Y, ctx.delta_time as f32 * f32::to_radians(rotator.speed));
    // }
    Ok(())
}

pub fn transform_propagate(ctx: &mut SystemContext, world: &mut World) -> Result<()> {
    
    // for (_, ())

    Ok(())
}