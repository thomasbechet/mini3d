use mini3d_runtime::{Runtime, RuntimeConfig};
use mini3d_stdlog::stdout::StdoutLogger;

// #[derive(Default, Clone)]
// struct SpawnSystem {
//     transform: NativeSingleViewMut<Transform>,
// }
//
// impl NativeExclusiveSystemInstance for SpawnSystem {
//     fn setup(&mut self, resolver: &mut Resolver) -> Result<(), ResolverError> {
//         self.transform.resolve(resolver, Transform::NAME)?;
//         println!("RESOLVED");
//         Ok(())
//     }
//     fn run(mut self, ctx: &mut Context) {
//         let e = Entity::create(ctx);
//         for i in 0..2 {
//             let e = Entity::create(ctx);
//             self.transform
//                 .add(e, Transform::from_translation(V3I32F16::ZERO));
//             if i == 1 {
//                 Entity::destroy(ctx, e);
//             }
//             self.transform
//                 .add(e, Transform::from_translation(V3I32F16::ZERO));
//         }
//         self.transform
//             .add(e, Transform::from_translation(V3I32F16::ZERO));
//         info!(ctx, "Spawned entity: {:?}", e);
//     }
// }
//
// #[derive(Default, Clone)]
// struct TestSystem {
//     transform: NativeSingleViewRef<Transform>,
//     query: Query,
// }
//
// impl NativeExclusiveSystemInstance for TestSystem {
//     fn setup(&mut self, resolver: &mut Resolver) -> Result<(), ResolverError> {
//         self.transform.resolve(resolver, Transform::NAME)?;
//         self.query.resolve(resolver).all(&[Transform::NAME])?;
//         Ok(())
//     }
//
//     fn run(self, ctx: &mut Context) {
//         for (i, e) in self.query.iter().enumerate() {
//             let transform = &self.transform[e];
//             info!(ctx, "{} {:?}", i, transform);
//         }
//         info!(ctx, "{:.3} {}", Time::delta(ctx), Time::frame(ctx));
//     }
// }

fn main() {
    let mut runtime = Runtime::new(RuntimeConfig::default().bootstrap(|api| {
    }));
    // let mut runtime = Runtime::new(RuntimeConfig::default().bootstrap(|ctx| {
    //     let spawn = System::create_native_exclusive::<SpawnSystem>(ctx, "SYS_SpawnSystem").unwrap();
    //     let test = System::create_native_exclusive::<TestSystem>(ctx, "SYS_TestSystem").unwrap();
    //     let propagate_transform = System::find(ctx, PropagateTransforms::NAME).unwrap();
    //     let free_fly = System::find(ctx, FreeFlySystem::NAME).unwrap();
    //     let start = SystemStage::find(ctx, SystemStage::START).unwrap();
    //     let stage = SystemStage::find(ctx, SystemStage::TICK).unwrap();
    //     let set = SystemSet::create(
    //         ctx,
    //         "SST_Root",
    //         SystemSet::new()
    //             .with("spawn", spawn, start, SystemOrder::default())
    //             .with("test", test, stage, SystemOrder::default())
    //             .with(
    //                 "propagate_transforms",
    //                 propagate_transform,
    //                 stage,
    //                 SystemOrder::default(),
    //             )
    //             .with("free_fly", free_fly, stage, SystemOrder::default()),
    //     )
    //     .unwrap();
    //     SystemSet::add(ctx, set);
    //     for (i, handle) in Resource::iter(ctx).enumerate() {
    //         let info = Resource::info(ctx, handle).unwrap();
    //         let ty_name = Resource::info(ctx, info.ty).unwrap().name;
    //         println!("[{}] {}   {}", i + 1, info.name, ty_name);
    //     }
    // }));
    runtime.set_logger(StdoutLogger);
    for _ in 0..10 {
        runtime.tick().expect("Simulation error");
    }
    println!("target_tps: {}", runtime.target_tps());
    println!("DONE");
}
