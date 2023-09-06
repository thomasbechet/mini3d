use mini3d::{
    ecs::{
        api::{ecs::ExclusiveECS, ExclusiveAPI},
        instance::SystemResult,
    },
    registry::system::ExclusiveSystem,
};

use crate::component::os::OS;

#[derive(Default)]
pub struct OSBootstrap;

impl ExclusiveSystem for OSBootstrap {
    const NAME: &'static str = "os_bootstrap";

    fn run(&self, _ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult {
        api.registry.components.add_static::<OS>("os")?;
        Ok(())
    }
}
