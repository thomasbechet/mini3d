use mini3d::{
    ecs::{
        api::{ecs::ExclusiveECS, ExclusiveAPI},
        instance::SystemResult,
        scheduler::Invocation,
    },
    registry::system::ExclusiveSystem,
};

use crate::{component::os::OS, system::initialize::OSInitialize};

#[derive(Default)]
pub struct OSBootstrap;

impl OSBootstrap {
    pub const NAME: &'static str = "os_bootstrap";
}

impl ExclusiveSystem for OSBootstrap {
    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult {
        api.registry.components.add_static::<OS>("os")?;
        api.registry.systems.add_static_exclusive::<OSInitialize>(
            OSInitialize::NAME,
            OSInitialize::NAME,
            Default::default(),
        )?;
        ecs.invoke(OSInitialize::NAME.into(), Invocation::Immediate)?;
        Ok(())
    }
}
