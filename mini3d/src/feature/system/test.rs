use glam::{Quat, Vec3};

use crate::{
    ecs::{
        context::ParallelContext,
        query::QueryId,
        system::{ParallelResolver, SystemResult},
    },
    feature::component::{common::rotator::Rotator, scene::transform::Transform},
    registry::{
        component::{Component, ComponentId},
        error::RegistryError,
        system::ParallelSystem,
    },
};

#[derive(Default)]
pub struct TestSystem {
    transform: StaticViewRef<Transform>,
    rotator: StaticViewMut<Rotator>,
    query: Query,
}

impl ParallelSystem for RotatorSystem {
    const NAME: &'static str = "rotator_system";

    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        self.transform = resolver.view(Transform::UID)?;
        Ok(())
    }

    fn run(&self, ctx: &mut ParallelContext) -> SystemResult {
        Ok(())
    }
}
