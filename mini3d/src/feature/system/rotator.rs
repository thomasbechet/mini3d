use glam::{Quat, Vec3};

use crate::{
    ecs::{context::ParallelContext, system::SystemResult},
    feature::component::{common::rotator::Rotator, scene::transform::Transform},
    registry::{
        component::{Component, ComponentId},
        error::RegistryError,
        system::{ParallelResolver, ParallelSystem},
    },
};

#[derive(Default)]
pub struct RotatorSystem {
    transform: ComponentId,
    rotator: ComponentId,
}

impl ParallelSystem for RotatorSystem {
    const NAME: &'static str = "rotator_system";

    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.transform = resolver.write(Transform::UID)?;
        self.rotator = resolver.read(Rotator::UID)?;
        Ok(())
    }

    fn run(&self, ctx: &mut ParallelContext) -> SystemResult {
        let mut transforms = ctx
            .scene
            .view_mut(self.transform)?
            .as_static::<Transform>()?;
        let rotators = ctx.scene.view(self.rotator)?.as_static::<Rotator>()?;
        for e in &ctx.scene.query(&[self.transform, self.rotator]) {
            transforms[e].rotation *= Quat::from_axis_angle(
                Vec3::Y,
                ctx.time.delta() as f32 * f32::to_radians(rotators[e].speed),
            );
        }
        Ok(())
    }
}
