use mini3d::{
    ecs::api::{ecs::ExclusiveECS, ExclusiveAPI},
    info,
    instance::Instance,
    registry::system::{ExclusiveSystem, SystemStage},
};
use mini3d_utils::stdout::StdoutLogger;

#[derive(Default)]
struct TestSystem;

impl ExclusiveSystem for TestSystem {
    fn run(&self, _: &mut ExclusiveECS, api: &mut ExclusiveAPI) {
        info!(api, "{:.3} {:.3}", api.time.global(), api.time.delta());
    }
}

fn main() {
    let mut instance = Instance::new(true);
    instance.set_logger_provider(StdoutLogger);
    instance
        .register_system::<TestSystem>("test_system", SystemStage::FIXED_UPDATE_60HZ)
        .unwrap();
    for _ in 0..60 {
        instance.progress(1.0 / 120.0).expect("Instance error");
    }
}
