use anyhow::Result;
use glam::{Quat, Vec3};

use crate::{feature::component::{rotator::Rotator, transform::Transform}, context::SystemContext};

pub fn run(ctx: &SystemContext) -> Result<()> {
    let world = ctx.world().active();
    let transforms = world.view_mut::<Transform>(Transform::UID)?;
    let rotators = world.view::<Rotator>(Rotator::UID)?;
    for e in &world.query(&[Transform::UID, Rotator::UID]) {
        transforms[e].rotation *= Quat::from_axis_angle(Vec3::Y, ctx.time().delta() as f32 * f32::to_radians(rotators[e].speed));
    }
    Ok(())
}