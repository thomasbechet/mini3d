use glam::{Quat, Vec3};

use crate::{
    ecs::{
        component::StaticComponent,
        context::ParallelContext,
        query::QueryId,
        system::{ParallelResolver, SystemResult},
    },
    feature::component::{common::rotator::Rotator, scene::transform::Transform},
    registry::{component::Component, error::RegistryError, system::ParallelSystem},
};

#[derive(Default)]
pub struct RotatorSystem {
    transform: StaticComponent<Transform>,
    rotator: StaticComponent<Rotator>,
    query: QueryId,
}

impl ParallelSystem for RotatorSystem {
    const NAME: &'static str = "rotator_system";

    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.transform = resolver.write(Transform::UID)?;
        self.rotator = resolver.read(Rotator::UID)?;
        self.query = resolver
            .query()
            .all(&[Transform::UID, Rotator::UID])?
            .build();
        Ok(())
    }

    fn run(&self, ctx: &mut ParallelContext) -> SystemResult {
        let mut transforms = ctx.scene.view_mut(self.transform)?;
        let rotators = ctx.scene.view(self.rotator)?;
        for e in ctx.scene.query(self.query) {
            transforms[e].rotation *= Quat::from_axis_angle(
                Vec3::Y,
                ctx.time.delta() as f32 * f32::to_radians(rotators[e].speed),
            );
        }
        Ok(())
    }
}
