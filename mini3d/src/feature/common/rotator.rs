use glam::{Quat, Vec3};
use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    api::{time::Time, Context},
    ecs::{
        error::ResolverError,
        query::Query,
        system::{ParallelSystem, SystemResolver},
        view::native::single::{NativeSingleViewMut, NativeSingleViewRef},
    },
};

use super::transform::Transform;

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct Rotator {
    pub speed: f32,
}

#[derive(Default, Clone)]
pub struct RotatorSystem {
    transform: NativeSingleViewMut<Transform>,
    rotator: NativeSingleViewRef<Rotator>,
    query: Query,
}

impl RotatorSystem {
    pub const NAME: &'static str = "SYS_Rotator";
}

impl ParallelSystem for RotatorSystem {
    fn setup(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        self.transform.resolve(resolver, Transform::NAME)?;
        self.rotator.resolve(resolver, Rotator::NAME)?;
        self.query
            .resolve(resolver)
            .all(&[Transform::NAME, Rotator::NAME])?;
        Ok(())
    }

    fn run(mut self, ctx: &Context) {
        for e in self.query.iter() {
            self.transform[e].rotation *= Quat::from_axis_angle(
                Vec3::Y,
                Time::delta(ctx) as f32 * f32::to_radians(self.rotator[e].speed),
            );
        }
    }
}
