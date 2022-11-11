use anyhow::Result;
use glam::{Quat, Vec3};
use hecs::World;

use crate::ecs::component::{transform::TransformComponent, rotator::RotatorComponent};

use super::{System, SystemContext};

pub struct RotatorSystem;

impl System for RotatorSystem {
    fn run(&self, ctx: &mut SystemContext, world: &mut World) -> Result<()> {
        for (_, (transform, rotator)) in world.query_mut::<(&mut TransformComponent, &RotatorComponent)>() {
            transform.rotation *= Quat::from_axis_angle(Vec3::Y, ctx.delta_time as f32 * f32::to_radians(rotator.speed));
        }
        Ok(())
    }
}