use glam::{Quat, Vec3};

use crate::{
    ecs::{
        api::{ecs::ParallelECS, ParallelAPI},
        instance::ParallelResolver,
        query::Query,
    },
    feature::component::{common::rotator::Rotator, scene::transform::Transform},
    registry::{component::StaticComponent, error::RegistryError, system::ParallelSystem},
};

#[derive(Default)]
pub struct RotatorSystem {
    transform: StaticComponent<Transform>,
    rotator: StaticComponent<Rotator>,
    query: Query,
}

impl RotatorSystem {
    const NAME: &'static str = "rotator_system";
}

impl ParallelSystem for RotatorSystem {
    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.transform = resolver.write(Transform::NAME.into())?;
        self.rotator = resolver.read(Rotator::NAME.into())?;
        self.query = resolver
            .query()
            .all(&[Transform::NAME.into(), Rotator::NAME.into()])?
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
