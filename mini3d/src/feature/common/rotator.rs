use glam::{Quat, Vec3};
use mini3d_derive::{Component, Reflect, Serialize};

use crate::{
    ecs::{
        api::{ecs::ParallelECS, ParallelAPI},
        instance::ParallelResolver,
        query::Query,
    },
    registry::{component::StaticComponent, error::RegistryError, system::ParallelSystem},
};

use super::transform::Transform;

#[derive(Default, Component, Serialize, Reflect, Clone)]
pub struct Rotator {
    pub speed: f32,
}

#[derive(Default)]
pub struct RotatorSystem {
    transform: StaticComponent<Transform>,
    rotator: StaticComponent<Rotator>,
    query: Query,
}

impl RotatorSystem {
    pub const NAME: &'static str = "rotator_system";
}

impl ParallelSystem for RotatorSystem {
    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.transform = resolver.write(Transform::NAME)?;
        self.rotator = resolver.read(Rotator::NAME)?;
        self.query = resolver
            .query()
            .all(&[Transform::NAME, Rotator::NAME])?
            .build();
        Ok(())
    }

    fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) {
        let mut transforms = ecs.view_mut(self.transform);
        let rotators = ecs.view(self.rotator);
        for e in ecs.query(self.query) {
            transforms[e].rotation *= Quat::from_axis_angle(
                Vec3::Y,
                api.time.delta() as f32 * f32::to_radians(rotators[e].speed),
            );
        }
    }
}
