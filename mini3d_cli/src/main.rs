use mini3d::{
    ecs::{
        api::{ecs::ExclusiveECS, ExclusiveAPI},
        instance::ExclusiveResolver,
        query::Query,
        scheduler::Invocation,
    },
    feature::component::scene::transform::Transform,
    info,
    instance::{Instance, InstanceFeatures},
    registry::{
        component::StaticComponent,
        error::RegistryError,
        system::{ExclusiveSystem, SystemStage},
    },
};
use mini3d_utils::stdout::StdoutLogger;

#[derive(Default)]
struct SpawnSystem;

impl ExclusiveSystem for SpawnSystem {
    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) {
        // let transforms = api
        //     .registry
        //     .components
        //     .add_static::<Transform>(Transform::NAME)
        //     .unwrap();
        let transforms: StaticComponent<Transform> =
            api.registry.components.find(Transform::NAME).unwrap();
        ecs.update_registry(&api.registry.components);
        let entity = ecs
            .add()
            .with(
                transforms,
                Transform::from_translation([0.0, 0.0, 0.0].into()),
            )
            .build();
        info!(api, "Spawned entity: {:?}", entity);
    }
}

#[derive(Default)]
struct TestSystem {
    transforms: StaticComponent<Transform>,
    transform_query: Query,
}

impl ExclusiveSystem for TestSystem {
    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.transforms = resolver.find(Transform::NAME)?;
        self.transform_query = resolver.query().all(&[Transform::NAME])?.build();
        Ok(())
    }

    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) {
        let transforms = ecs.view(self.transforms);
        // for transform in transforms.iter() {
        //     info!(api, "{:?}", transform);
        // }
        for (i, e) in ecs.query(self.transform_query).enumerate() {
            let transform = &transforms[e];
            info!(api, "{} {:?}", i, transform);
        }
        info!(api, "{:.3} {:.3}", api.time.global(), api.time.delta());
    }
}

fn main() {
    let mut instance = Instance::new(InstanceFeatures::all());
    instance.set_logger_provider(StdoutLogger);
    instance
        .register_system::<TestSystem>("test_system", SystemStage::FIXED_UPDATE_60HZ)
        .unwrap();
    instance
        .register_system::<SpawnSystem>("spawn_system", "startup")
        .unwrap();
    instance.invoke("startup", Invocation::NextFrame).unwrap();
    for _ in 0..60 {
        instance.progress(1.0 / 120.0).expect("Instance error");
    }
}
