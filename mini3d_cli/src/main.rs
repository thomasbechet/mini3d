use mini3d::{
    ecs::{
        api::{context::Context, ecs::ECS, registry::ComponentRegistry, time::Time},
        instance::ExclusiveResolver,
        query::QueryId,
        scheduler::Invocation,
    },
    engine::{Engine, EngineFeatures},
    feature::common::transform::Transform,
    info,
    registry::{
        component::StaticComponentType,
        error::RegistryError,
        system::{ExclusiveSystem, SystemStage},
    },
};
use mini3d_utils::stdout::StdoutLogger;

#[derive(Default)]
struct SpawnSystem;

impl ExclusiveSystem for SpawnSystem {
    fn run(&self, ecs: &mut ECS, ctx: &mut Context) {
        // let transforms = ctx
        //     .registry
        //     .components
        //     .add_static::<Transform>(Transform::NAME)
        //     .unwrap();
        let transforms: StaticComponentType<Transform> =
            ComponentRegistry::find(ctx, Transform::NAME).unwrap();
        let entity = ecs
            .create()
            .with(
                transforms,
                Transform::from_translation([0.0, 0.0, 0.0].into()),
            )
            .build();
        info!(ctx, "Spawned entity: {:?}", entity);
    }
}

#[derive(Default)]
struct TestSystem {
    transforms: StaticComponentType<Transform>,
    transform_query: QueryId,
}

impl ExclusiveSystem for TestSystem {
    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        self.transforms = resolver.find(Transform::NAME)?;
        self.transform_query = resolver.query().all(&[Transform::NAME])?.build();
        Ok(())
    }

    fn run(&self, ecs: &mut ECS, ctx: &mut Context) {
        // let ui = ecs.add().with(self.ui, UI::new()).build();
        // ecs.add().with(self.button, UIButton::new(ui)).build();

        let transforms = ecs.view(self.transforms);
        // for transform in transforms.iter() {
        //     info!(ctx, "{:?}", transform);
        // }
        for (i, e) in ecs.query(self.transform_query).enumerate() {
            let transform = &transforms[e];
            info!(ctx, "{} {:?}", i, transform);
        }
        info!(ctx, "{:.3} {:.3}", Time::global(ctx), Time::delta(ctx));
    }
}

fn main() {
    let mut engine = Engine::new(EngineFeatures::all());
    engine.set_logger_provider(StdoutLogger);
    engine
        .register_system::<TestSystem>("test_system", SystemStage::FIXED_UPDATE_60HZ)
        .unwrap();
    engine
        .register_system::<SpawnSystem>("spawn_system", "startup")
        .unwrap();
    engine.invoke("startup", Invocation::NextFrame).unwrap();
    engine.progress(1.0 / 120.0).expect("Instance error");
}
