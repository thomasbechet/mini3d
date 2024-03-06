use mini3d_derive::fixed;
use mini3d_runtime::{
    api::API, container::ComponentId, entity::Entity, event::EventStage, field::{Field, FieldType}, fixed::I32F16, info, quat::QI32F16, query::Query, vec::V3I32F16, warn, Runtime, RuntimeConfig
};
use mini3d_stdlog::stdout::StdoutLogger;

pub struct Transform {
    _id: ComponentId,
    pub pos: Field<V3I32F16>,
    pub rot: Field<QI32F16>,
    pub sca: Field<V3I32F16>,
}

impl Transform {
    pub fn register(api: &mut API) -> Self {
        api.register(
            "transform",
            &[
                V3I32F16::named("pos"),
                QI32F16::named("rot"),
                V3I32F16::named("sca"),
            ],
        )
        .unwrap();
        Self::meta(api)
    }
    pub fn meta(api: &API) -> Self {
        let id = api.find_component("transform").unwrap();
        Self {
            _id: id,
            pos: api.find_field(id, "pos").unwrap(),
            rot: api.find_field(id, "rot").unwrap(),
            sca: api.find_field(id, "sca").unwrap(),
        }
    }
    pub fn id(&self) -> ComponentId {
        self._id
    }
    pub fn add(&self, api: &mut API, e: Entity, pos: V3I32F16) {
        api.add_default(e, self.id());
        api.write(e, self.pos, pos);
    }
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
    let transform = Transform::meta(api);
    let query = Query::default().all(&[transform.id()]);
    info!(api, "hello world");
    let e = api.entities().next().unwrap();
    let mut pos = api.read(e, transform.pos).unwrap();
    info!(api, "{:?}", pos);
    warn!(api, "WTF");
    pos.x += fixed!(0.5i32f16);
    api.write(e, transform.pos, pos);
    for e in api.query_entities(&query) {
        let pos = api.read(e, transform.pos).unwrap();
        info!(api, "t {:?}", pos);
    }
}

fn main() {
    let mut runtime = Runtime::new(RuntimeConfig::default().bootstrap(|api| {
        let transform = Transform::register(api);
        let tick_stage = api.event_stage(EventStage::Tick).unwrap();
        api.add_system("my_system", tick_stage, Default::default(), my_system)
            .unwrap();
        let e = api.create();
        api.add_default(e, transform.id());
        let e = api.create();
        api.add_default(e, transform.id());
        transform.add(api, e, V3I32F16::ONE);
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
    for _ in 0..5{
        runtime.tick().expect("Simulation error");
    }
    println!("target_tps: {}", runtime.target_tps());
    println!("DONE");
}
