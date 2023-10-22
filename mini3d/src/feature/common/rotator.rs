use glam::{Quat, Vec3};
use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    ecs::{
        api::{ecs::ECS, time::Time, Context},
        error::ResolverError,
        query::QueryId,
        system::{ParallelResolver, ParallelSystem},
    },
    feature::core::component::ComponentId,
};

use super::transform::Transform;

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct Rotator {
    pub speed: f32,
}

#[derive(Default)]
pub struct RotatorSystem {
    transform: ComponentId,
    rotator: ComponentId,
    query: QueryId,
}

impl RotatorSystem {
    pub const NAME: &'static str = "rotator_system";
}

impl ParallelSystem for RotatorSystem {
    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), ResolverError> {
        self.transform = resolver.write(Transform::NAME)?;
        self.rotator = resolver.read(Rotator::NAME)?;
        self.query = resolver
            .query()
            .all(&[Transform::NAME, Rotator::NAME])?
            .build();
        Ok(())
    }

    fn run(&self, ctx: &Context) {
        let mut transforms = ECS::view_mut(ctx, self.transform);
        let rotators = ECS::view(ctx, self.rotator);
        for e in ECS::query(ctx, self.query) {
            transforms[e].rotation *= Quat::from_axis_angle(
                Vec3::Y,
                Time::delta(ctx) as f32 * f32::to_radians(rotators[e].speed),
            );
        }
    }
}
