use mini3d::{
    api::{activity::Activity, resource::Resource, time::Time, Context},
    ecs::{
        entity::Entity,
        error::ResolverError,
        query::Query,
        system::{ExclusiveSystem, SystemResolver},
        view::native::single::{NativeSingleViewMut, NativeSingleViewRef},
    },
    engine::{Engine, EngineConfig},
    feature::{
        common::{
            free_fly::FreeFlySystem,
            transform::{PropagateTransforms, Transform},
        },
        ecs::system::{System, SystemOrder, SystemSet, SystemStage},
    },
    info,
};
use mini3d_utils::stdout::StdoutLogger;

#[derive(Default, Clone)]
struct SpawnSystem {
    transform: NativeSingleViewMut<Transform>,
}

impl ExclusiveSystem for SpawnSystem {
    fn setup(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        self.transform.resolve(resolver, Transform::NAME)?;
        println!("RESOLVED");
        Ok(())
    }
    fn run(mut self, ctx: &mut Context) {
        let e = Entity::create(ctx);
        for i in 0..2 {
            let e = Entity::create(ctx);
            self.transform
                .add(e, Transform::from_translation([0.0, 0.0, 0.0].into()));
            if i == 5 {
                Entity::destroy(ctx, e);
            }
            self.transform
                .add(e, Transform::from_translation([0.0, 0.0, 0.0].into()));
        }
        self.transform
            .add(e, Transform::from_translation([0.0, 0.0, 0.0].into()));
        info!(ctx, "Spawned entity: {:?}", e);
    }
}

#[derive(Default, Clone)]
struct TestSystem {
    transform: NativeSingleViewRef<Transform>,
    query: Query,
}

impl ExclusiveSystem for TestSystem {
    fn setup(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
        self.transform.resolve(resolver, Transform::NAME)?;
        self.query.resolve(resolver).all(&[Transform::NAME])?;
        Ok(())
    }

    fn run(self, ctx: &mut Context) {
        for (i, e) in self.query.iter().enumerate() {
            let transform = &self.transform[e];
            info!(ctx, "{} {:?}", i, transform);
        }
        info!(ctx, "{:.3} {:.3}", Time::global(ctx), Time::delta(ctx));
    }
}

fn main() {
    let mut engine = Engine::new(EngineConfig::default().bootstrap(|ctx| {
        let spawn = System::create_native_exclusive::<SpawnSystem>(ctx, "SYS_SpawnSystem").unwrap();
        let test = System::create_native_exclusive::<TestSystem>(ctx, "SYS_TestSystem").unwrap();
        let propagate_transform = System::find(ctx, PropagateTransforms::NAME).unwrap();
        let free_fly = System::find(ctx, FreeFlySystem::NAME).unwrap();
        let stage = SystemStage::find(ctx, SystemStage::UPDATE).unwrap();
        let set = SystemSet::create(
            ctx,
            "SST_Root",
            SystemSet::new()
                .with("spawn", spawn, stage, SystemOrder::default())
                .with("test", test, stage, SystemOrder::default())
                .with(
                    "propagate_transforms",
                    propagate_transform,
                    stage,
                    SystemOrder::default(),
                )
                .with("free_fly", free_fly, stage, SystemOrder::default()),
        )
        .unwrap();
        Activity::add_system_set(ctx, Activity::active(ctx), set);
        for (i, handle) in Resource::iter(ctx).enumerate() {
            let info = Resource::info(ctx, handle).unwrap();
            println!("[{}] {}   {}", i + 1, info.key, info.ty_name);
        }
    }));
    engine.set_logger(StdoutLogger);
    engine.progress(1.0 / 120.0).expect("Instance error");
    println!("DONE");
}
