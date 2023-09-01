use glam::{Quat, Vec3};

use crate::{
    ecs::{
        api::{ecs::ParallelECS, ParallelAPI},
        query::Query,
        system::{ParallelResolver, SystemResult},
    },
    feature::component::{common::rotator::Rotator, scene::transform::Transform},
    registry::{
        component::{Component, StaticComponent},
        error::RegistryError,
        system::ParallelSystem,
    },
};

#[derive(Default)]
pub struct RotatorSystem {
    transform: StaticComponent<Transform>,
    rotator: StaticComponent<Rotator>,
    query: Query,
}

impl ParallelSystem for RotatorSystem {
    const NAME: &'static str = "rotator_system";

    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.transform = resolver.write(Transform::UID)?;
        self.rotator = resolver.read(Rotator::UID)?;
        self.query = resolver
            .query()
            .all(&[Transform::UID, Rotator::UID])?
            .build();
        Ok(())
    }

    fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) -> SystemResult {
        let mut transforms = ecs.view_mut(self.transform)?;
        let rotators = ecs.view(self.rotator)?;
        for e in ecs.query(self.query) {
            transforms[e].rotation *= Quat::from_axis_angle(
                Vec3::Y,
                api.time.delta() as f32 * f32::to_radians(rotators[e].speed),
            );
        }
        Ok(())
    }
}
