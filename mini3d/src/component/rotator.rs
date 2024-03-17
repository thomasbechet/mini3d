use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    ecs::{
        context::{Context, Time},
        query::Query,
        view::native::single::{NativeSingleViewMut, NativeSingleViewRef},
    },
    math::{
        fixed::{FixedPoint, TrigFixedPoint, I32F16},
        quat::Q,
        vec::V3,
    },
};

use super::{transform::Transform, SystemConfig, SystemParam};

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct Rotator {
    pub speed: I32F16,
}

pub const ROTATOR_SYSTEM_CONFIG: SystemConfig = SystemConfig::new(&[
    SystemParam::ViewMut(Transform::NAME),
    SystemParam::ViewRef(Rotator::NAME),
    SystemParam::Query {
        all: &[Transform::NAME, Rotator::NAME],
        any: &[],
        not: &[],
    },
]);

fn rotator_system(
    ctx: &Context,
    mut v_transform: NativeSingleViewMut<Transform>,
    v_rotator: NativeSingleViewRef<Rotator>,
    query: Query,
) {
    for e in query.iter() {
        v_transform[e].rotation *= Q::from_axis_angle(
            V3::Y,
            I32F16::cast(Time::delta(ctx)) * v_rotator[e].speed.to_radians(),
        );
    }
}
