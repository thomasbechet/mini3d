use anyhow::Result;
use glam::{Quat, Vec3};

use crate::{feature::component::{rotator::Rotator, transform::Transform}, context::SystemContext};

pub fn run(ctx: &SystemContext) -> Result<()> {
    let transforms = ctx.world().view_mut::<Transform>(Transform::UID)?;
    let rotators = ctx.world().view::<Rotator>(Rotator::UID)?;
    for e in &ctx.world().query(&[Transform::UID, Rotator::UID]) {
        transforms[e].rotation *= Quat::from_axis_angle(Vec3::Y, ctx.delta_time() as f32 * f32::to_radians(rotators[e].speed));
    }
    Ok(())
}