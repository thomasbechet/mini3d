use glam::{Quat, Vec3};

use crate::{
    context::ExclusiveSystemContext,
    ecs::system::SystemResult,
    feature::component::{common::rotator::Rotator, scene::transform::Transform},
    registry::component::Component,
};

pub fn run(ctx: &mut ExclusiveSystemContext) -> SystemResult {
    let scene = ctx.scene.active();
    let mut transforms = scene.static_view_mut::<Transform>(Transform::UID)?;
    let rotators = scene.static_view::<Rotator>(Rotator::UID)?;
    for e in &scene.query(&[Transform::UID, Rotator::UID]) {
        transforms[e].rotation *= Quat::from_axis_angle(
            Vec3::Y,
            ctx.time.delta() as f32 * f32::to_radians(rotators[e].speed),
        );
    }
    Ok(())
}
