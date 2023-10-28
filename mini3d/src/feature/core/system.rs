use mini3d_derive::{Reflect, Serialize};

use crate::{
    api::Context,
    ecs::{
        error::ResolverError,
        system::{
            AnyNativeExclusiveSystemInstance, AnyNativeParallelSystemInstance, ExclusiveResolver,
            ExclusiveSystem, ExclusiveSystemInstance, ParallelResolver, ParallelSystem,
            ParallelSystemInstance, SystemInstance,
        },
    },
    resource::handle::{ReferenceResolver, ResourceHandle},
    utils::string::AsciiArray,
};

use super::resource::Resource;

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
            fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), ResolverError> {
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
            fn resolve(
                &mut self,
                resolver: &mut ParallelResolver<'_>,
            ) -> Result<(), ResolverError> {
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

#[derive(Default, Debug, Serialize, Reflect, Clone)]
pub struct System {
    pub(crate) kind: SystemKind,
}

impl Resource for System {
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver) {
        match self {
            Self {
                kind: SystemKind::Script { script },
            } => script.resolve(resolver),
            _ => {}
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct SystemStage {
    pub(crate) name: AsciiArray<32>,
    pub(crate) periodic: Option<f64>,
}

impl SystemStage {
    pub const UPDATE: &'static str = "update";
    pub const FIXED_UPDATE_60HZ: &'static str = "fixed_update_60hz";
}

impl Resource for SystemStage {}

#[derive(Default)]
pub struct SystemOrder;

pub struct SystemSetEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) system: ResourceHandle,
    pub(crate) stage: ResourceHandle,
    pub(crate) order: SystemOrder,
}

pub struct SystemSet(pub(crate) Vec<SystemSetEntry>);

impl Resource for SystemSet {
    fn resolve_references(&mut self, resolver: &mut ReferenceResolver) {
        for system in self.0.iter_mut() {
            system.system.resolve(resolver);
            system.stage.resolve(resolver);
        }
    }
}
