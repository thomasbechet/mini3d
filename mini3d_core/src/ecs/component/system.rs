use alloc::{boxed::Box, vec::Vec};
use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        context::Context,
        entity::Entity,
        error::ResolverError,
        scheduler::Invocation,
        system::{
            AnyNativeExclusiveSystemInstance, AnyNativeParallelSystemInstance, ExclusiveSystem,
            ExclusiveSystemInstance, ParallelSystem, ParallelSystemInstance, SystemInstance,
            SystemResolver,
        },
    },
    math::fixed::U32F16,
    utils::string::AsciiArray,
};

use super::Component;

pub(crate) trait SystemReflection {
    fn create_instance(&self) -> SystemInstance;
}

struct NativeExclusiveSystemReflection<S: ExclusiveSystem> {
    _phantom: core::marker::PhantomData<S>,
}

impl<S: ExclusiveSystem> SystemReflection for NativeExclusiveSystemReflection<S> {
    fn create_instance(&self) -> SystemInstance {
        struct InstanceHolder<S: ExclusiveSystem> {
            data: S,
        }
        impl<S: ExclusiveSystem> AnyNativeExclusiveSystemInstance for InstanceHolder<S> {
            fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), ResolverError> {
                self.data = Default::default();
                self.data.setup(resolver)
            }
            fn run(&self, ctx: &mut Context) {
                S::run(self.data.clone(), ctx);
            }
        }
        SystemInstance::Exclusive(ExclusiveSystemInstance::Native(Box::new(InstanceHolder {
            data: S::default(),
        })))
    }
}

struct NativeParallelSystemReflection<S: ParallelSystem> {
    _phantom: core::marker::PhantomData<S>,
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
                S::run(self.system.clone(), ctx);
            }
        }
        SystemInstance::Parallel(ParallelSystemInstance::Native(Box::new(InstanceHolder {
            system: S::default(),
        })))
    }
}

#[derive(Serialize)]
pub(crate) enum SystemKind {
    Native(#[serialize(skip)] Box<dyn SystemReflection>),
    Script,
}

impl Default for SystemKind {
    fn default() -> Self {
        Self::Script
    }
}

impl Default for Box<dyn SystemReflection> {
    fn default() -> Self {
        panic!("Invalid deserialize for native system reflection")
    }
}

#[derive(Default, Serialize, Reflect)]
pub struct System {
    pub(crate) kind: SystemKind,
}

impl System {
    pub const NAME: &'static str = "system";

    pub fn native_exclusive<S: ExclusiveSystem>() -> Self {
        let reflection = NativeExclusiveSystemReflection::<S> {
            _phantom: core::marker::PhantomData,
        };
        Self {
            kind: SystemKind::Native(Box::new(reflection)),
        }
    }

    pub fn native_parallel<S: ParallelSystem>() -> Self {
        let reflection = NativeParallelSystemReflection::<S> {
            _phantom: core::marker::PhantomData,
        };
        Self {
            kind: SystemKind::Native(Box::new(reflection)),
        }
    }

    pub fn script() -> Self {
        Self {
            kind: SystemKind::Script,
        }
    }

    pub fn enable(ctx: &mut Context, set: Entity) {}

    pub fn disable(ctx: &mut Context, set: Entity) {}
}

impl Component for System {}

#[derive(Default, Clone, Reflect, Serialize)]
pub struct SystemStage {
    pub(crate) periodic: Option<U32F16>,
}

impl SystemStage {
    pub const NAME: &'static str = "system_stage";
    pub const START: &'static str = "start";
    pub const TICK: &'static str = "tick";

    pub fn periodic(periodic: U32F16) -> Self {
        Self {
            periodic: Some(periodic),
        }
    }

    pub fn invoke(ctx: &mut Context, stage: Entity, invocation: Invocation) {
        ctx.scheduler.invoke(stage, invocation)
    }
}

impl Component for SystemStage {}

#[derive(Default, Serialize, Reflect)]
pub struct SystemOrder {}

#[derive(Default, Reflect, Serialize)]
pub struct SystemSetEntry {
    pub(crate) name: AsciiArray<32>,
    pub(crate) system: Entity,
    pub(crate) stage: Entity,
    pub(crate) order: SystemOrder,
}

#[derive(Default, Reflect, Serialize)]
pub struct SystemSet(pub(crate) Vec<SystemSetEntry>);

impl SystemSet {
    pub const NAME: &'static str = "system_set";

    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn with(mut self, name: &str, system: Entity, stage: Entity, order: SystemOrder) -> Self {
        if let Some(entry) = self.0.iter_mut().find(|e| e.name.as_str() == name) {
            entry.system = system;
            entry.stage = stage;
            entry.order = order;
        } else {
            self.0.push(SystemSetEntry {
                name: AsciiArray::from(name),
                system,
                stage,
                order,
            });
        }
        self
    }

    pub fn enable(&mut self, ctx: &mut Context) {}

    pub fn disable(&mut self, ctx: &mut Context) {}
}

impl Component for SystemSet {
    fn resolve_entities(
        &mut self,
        resolver: &mut super::EntityResolver,
    ) -> Result<(), super::ComponentError> {
        Ok(())
    }
    fn on_added(
        &mut self,
        entity: Entity,
        ctx: super::ComponentContext,
    ) -> Result<(), super::ComponentError> {
        Ok(())
    }
    fn on_removed(
        &mut self,
        entity: Entity,
        ctx: super::ComponentContext,
    ) -> Result<(), super::ComponentError> {
        Ok(())
    }
}
