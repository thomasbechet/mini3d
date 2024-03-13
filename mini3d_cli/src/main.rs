use mini3d_derive::fixed;
use mini3d_runtime::{
    api::API, component::{hierarchy::Hierarchy, transform::Transform}, field::{Field, FieldType}, fixed::I32F16, info, quat::QI32F16, query::Query, vec::V3I32F16, Runtime, RuntimeConfig
};
use mini3d_stdlog::stdout::StdoutLogger;

fn on_transform_added(api: &mut API) {
    info!(api, "TRANSFORM ADDED")
}
fn on_transform_removed(api: &mut API) {
    info!(api, "TRANSFORM REMOVED")
}

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

fn my_system(api: &mut API) {
    info!(api, "my_system");
    let transform = Transform::meta(api);
    let hierarchy = Hierarchy::meta(api);
    let my_tag = api.find_component("my_tag").unwrap();
    let query = Query::default().all(&[transform.id()]);
    info!(api, "hello world");
    let e = api.entities().next().unwrap();
    let mut pos = api.read(e, transform.translation).unwrap();
    info!(api, "{:?}", pos);
    pos.x += fixed!(0.5i32f16);
    api.write(e, transform.translation, pos);
    for e in api.entities() {
        api.dump(e);
    }
}

fn on_start(api: &mut API) {
    let transform = Transform::register(api);
    let hierarchy = Hierarchy::register(api);
    let my_tag = api.register_tag("my_tag").unwrap();
    let e = api.create();
    transform.add_from_translation(api, e, V3I32F16::ONE);
    api.add_default(e, hierarchy.id());
    api.debug_sched();
    api.add_default(e, my_tag);
    let e = api.create();
    api.add_default(e, my_tag);
}

fn main() {
    let mut runtime = Runtime::new(RuntimeConfig::default().bootstrap(|api| {
        api.create_system("my_system", api.tick_stage(), Default::default(), my_system)
            .unwrap();
        api.create_system("start_system", api.start_stage(), Default::default(), on_start).unwrap();
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
    for _ in 0..5 {
        runtime.tick().expect("Simulation error");
    }
    println!("target_tps: {}", runtime.target_tps());
    println!("DONE");
}
