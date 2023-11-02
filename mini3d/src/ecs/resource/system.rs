use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{
    api::Context,
    define_resource_handle,
    ecs::{
        error::ResolverError,
        system::{
            AnyNativeExclusiveSystemInstance, AnyNativeParallelSystemInstance, ExclusiveSystem,
            ExclusiveSystemInstance, ParallelSystem, ParallelSystemInstance, SystemInstance,
            SystemResolver,
        },
    },
    feature::common::script::ScriptHandle,
    resource::handle::{ReferenceResolver, ResourceHandle},
    utils::string::AsciiArray,
};

pub(crate) trait SystemReflection {
    fn create_instance(&self) -> SystemInstance;
}

struct NativeExclusiveSystemReflection<S: ExclusiveSystem> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: ExclusiveSystem> SystemReflection for NativeExclusiveSystemReflection<S> {
    fn create_instance(&self) -> SystemInstance {
        struct InstanceHolder<S: ExclusiveSystem> {
            system: S,
        }
        impl<S: ExclusiveSystem> AnyNativeExclusiveSystemInstance for InstanceHolder<S> {
            fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
                self.system.setup(resolver)
            }
            fn run(&self, ctx: &mut Context) {
                self.system.run(ctx);
            }
        }
        SystemInstance::Exclusive(ExclusiveSystemInstance::Native(Box::new(InstanceHolder {
            system: S::default(),
        })))
    }
}

struct NativeParallelSystemReflection<S: ParallelSystem> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: ParallelSystem> SystemReflection for NativeParallelSystemReflection<S> {
    fn create_instance(&self) -> SystemInstance {
        struct InstanceHolder<S: ParallelSystem> {
            system: S,
        }
        impl<S: ParallelSystem> AnyNativeParallelSystemInstance for InstanceHolder<S> {
            fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
                self.system.setup(resolver)
            }
            fn run(&self, ctx: &Context) {
                self.system.run(ctx);
            }
        }
        SystemInstance::Parallel(ParallelSystemInstance::Native(Box::new(InstanceHolder {
            system: S::default(),
        })))
    }
}

pub(crate) enum SystemKind {
    Native {
        reflection: Box<dyn SystemReflection>,
    },
    Script {
        script: ResourceHandle,
    },
}

#[derive(Default, Serialize, Debug, Reflect, Clone)]
pub struct System {
    #[serialize(skip)]
    pub(crate) kind: SystemKind,
}

define_resource_handle!(SystemHandle);

impl System {
    pub const NAME: &'static str = "_system";

    pub fn native_exclusive<S: ExclusiveSystem>() -> Self {
        let reflection = NativeExclusiveSystemReflection::<S> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            kind: SystemKind::Native {
                reflection: Box::new(reflection),
            },
        }
    }

    pub fn native_parallel<S: ParallelSystem>() -> Self {
        let reflection = NativeParallelSystemReflection::<S> {
            _phantom: std::marker::PhantomData,
        };
        Self {
            kind: SystemKind::Native {
                reflection: Box::new(reflection),
            },
        }
    }

    pub fn script(script: ScriptHandle) -> Self {
        Self {
            kind: SystemKind::Script { script },
        }
    }
}

impl ResourceData for System {
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver) {
        match self {
            Self {
                kind: SystemKind::Script { script },
            } => script.resolve(resolver),
            _ => {}
        }
    }
}

#[derive(Default, Debug, Clone, Reflect, Serialize, Resource)]
pub struct SystemStage {
    pub(crate) name: AsciiArray<32>,
    pub(crate) periodic: Option<f64>,
}

define_resource_handle!(SystemStageHandle);

impl SystemStage {
    pub const NAME: &'static str = "_system_stage";
    pub const UPDATE: &'static str = "update";
    pub const FIXED_UPDATE_60HZ: &'static str = "fixed_update_60hz";
}

#[derive(Default, Serialize, Reflect)]
pub struct SystemOrder {}

#[derive(Default, Reflect, Serialize)]
pub struct SystemSetEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) system: SystemHandle,
    pub(crate) stage: SystemStageHandle,
    pub(crate) order: SystemOrder,
}

#[derive(Default, Reflect, Serialize)]
pub struct SystemSet(pub(crate) Vec<SystemSetEntry>);

define_resource_handle!(SystemSetHandle);

impl SystemSet {
    pub const NAME: &'static str = "_system_set";

    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with(
        mut self,
        name: &str,
        system: SystemHandle,
        stage: SystemStageHandle,
        order: SystemOrder,
    ) -> Self {
        if let Some(entry) = self.0.iter_mut().find(|e| e.name == name) {
            entry.system = system;
            entry.stage = stage;
            entry.order = order;
            return self;
        } else {
            self.0.push(SystemSetEntry {
                name: AsciiArray::from_str(name),
                system,
                stage,
                order,
            });
            return self;
        }
    }
}

impl ResourceData for SystemSet {
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver) {
        for system in self.0.iter_mut() {
            system.system.resolve(resolver);
            system.stage.resolve(resolver);
        }
    }
}
