use mini3d_derive::{Reflect, Resource, Serialize};

use crate::{
    ecs::{
        api::context::Context,
        error::ResolverError,
        system::{
            AnyStaticExclusiveSystemInstance, AnyStaticParallelSystemInstance, ExclusiveResolver,
            ExclusiveSystem, ExclusiveSystemInstance, ParallelResolver, ParallelSystem,
            ParallelSystemInstance, SystemInstance,
        },
    },
    resource::handle::ResourceRef,
};

use super::resource_type::Resource;

pub(crate) trait AnySystemReflection {
    fn create_instance(&self) -> SystemInstance;
}

struct StaticExclusiveSystemReflection<S: ExclusiveSystem> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: ExclusiveSystem> AnySystemReflection for StaticExclusiveSystemReflection<S> {
    fn create_instance(&self) -> SystemInstance {
        struct InstanceHolder<S: ExclusiveSystem> {
            system: S,
        }
        impl<S: ExclusiveSystem> AnyStaticExclusiveSystemInstance for InstanceHolder<S> {
            fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), ResolverError> {
                self.system.setup(resolver)
            }
            fn run(&self, ctx: &mut Context) {
                self.system.run(ctx);
            }
        }
        SystemInstance::Exclusive(ExclusiveSystemInstance::Static(Box::new(InstanceHolder {
            system: S::default(),
        })))
    }
}

struct StaticParallelSystemReflection<S: ParallelSystem> {
    _phantom: std::marker::PhantomData<S>,
}

impl<S: ParallelSystem> AnySystemReflection for StaticParallelSystemReflection<S> {
    fn create_instance(&self) -> SystemInstance {
        struct InstanceHolder<S: ParallelSystem> {
            system: S,
        }
        impl<S: ParallelSystem> AnyStaticParallelSystemInstance for InstanceHolder<S> {
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
        SystemInstance::Parallel(ParallelSystemInstance::Static(Box::new(InstanceHolder {
            system: S::default(),
        })))
    }
}

#[derive(Default)]
pub struct SystemOrder;

pub(crate) enum SystemKind {
    Native,
    Script { script: ResourceRef },
}

#[derive(Default, Debug, Serialize, Reflect, Clone)]
pub struct System {
    pub(crate) stage: ResourceRef,
    pub(crate) reflection: Box<dyn AnySystemReflection>,
    pub(crate) order: SystemOrder,
    pub(crate) active_by_default: bool,
}

impl Resource for System {}
