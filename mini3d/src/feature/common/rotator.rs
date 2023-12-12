use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    api::{time::Time, Context},
    ecs::{
        error::ResolverError,
        query::Query,
        system::{ParallelSystem, SystemResolver},
        view::native::single::{NativeSingleViewMut, NativeSingleViewRef},
    },
    math::{
        fixed::{FixedPoint, TrigFixedPoint, I32F16},
        quat::Q,
        vec::V3,
    },
};

use super::transform::Transform;

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct Rotator {
    pub speed: I32F16,
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
            self.transform[e].rotation *= Q::from_axis_angle(
                V3::Y,
                Time::delta(ctx).cast::<I32F16>() * self.rotator[e].speed.to_radians(),
            );
        }
    }
}
