use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        api::context::Context,
        error::ResolverError,
        system::{
            AnyNativeExclusiveSystemInstance, AnyNativeParallelSystemInstance, ExclusiveResolver,
            ExclusiveSystem, ExclusiveSystemInstance, ParallelResolver, ParallelSystem,
            ParallelSystemInstance, SystemInstance,
        },
    },
    resource::handle::ResourceRef,
};

use super::resource_type::Resource;

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
        script: ResourceRef,
    },
}

#[derive(Default, Debug, Serialize, Reflect, Clone)]
pub struct System {
    pub(crate) kind: SystemKind,
}

impl Resource for System {}
