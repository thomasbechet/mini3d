use alloc::{boxed::Box, vec::Vec};
use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        context::Context,
        entity::Entity,
        error::ResolverError,
        scheduler::{Invocation, SystemStageKey},
        system::{
            AnyNativeExclusiveSystemInstance, AnyNativeParallelSystemInstance, ExclusiveSystem,
            ExclusiveSystemInstance, ParallelSystem, ParallelSystemInstance, SystemInstance,
            SystemResolver,
        },
        view::native::single::NativeSingleViewMut,
    },
    math::fixed::U32F16,
    utils::{slotmap::Key, string::AsciiArray},
};

use super::{Component, ComponentContext, ComponentError, ComponentStorage, EntityResolver};

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
            fn run(&mut self, ctx: &Context) {
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
}

impl Component for System {
    const STORAGE: ComponentStorage = ComponentStorage::Single;
}

#[derive(Default, Clone, Reflect, Serialize)]
pub struct SystemStage {
    pub(crate) periodic: Option<U32F16>,
    #[serialize(skip)]
    pub(crate) key: SystemStageKey,
}

impl SystemStage {
    pub const NAME: &'static str = "system_stage";
    pub const START: &'static str = "start";
    pub const TICK: &'static str = "tick";

    pub fn periodic(periodic: U32F16) -> Self {
        Self {
            periodic: Some(periodic),
            key: SystemStageKey::null(),
        }
    }

    pub fn invoke(&self, ctx: &mut Context, invocation: Invocation) {
        ctx.scheduler.invoke(self.key, invocation)
    }
}

impl Component for SystemStage {
    const STORAGE: ComponentStorage = ComponentStorage::Single;
    fn on_added(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {}
    fn on_removed(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {}
}

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
pub struct SystemSet {
    pub(crate) entries: Vec<SystemSetEntry>,
    #[serialize(skip)]
    pub(crate) key: SystemSetKey,
}

impl SystemSet {
    pub const NAME: &'static str = "system_set";

    pub fn new() -> Self {
        Default::default()
    }

    pub fn with(mut self, name: &str, system: Entity, stage: Entity, order: SystemOrder) -> Self {
        if let Some(entry) = self.entries.iter_mut().find(|e| e.name.as_str() == name) {
            entry.system = system;
            entry.stage = stage;
            entry.order = order;
        } else {
            self.entries.push(SystemSetEntry {
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
    const STORAGE: ComponentStorage = ComponentStorage::Single;
    fn resolve_entities(&mut self, resolver: &mut EntityResolver) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_added(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {
        Ok(())
    }
    fn on_removed(&mut self, entity: Entity, ctx: ComponentContext) -> Result<(), ComponentError> {
        Ok(())
    }
}

trait SystemParam {}

impl<C: Component> SystemParam for NativeSingleViewMut<C> {}

trait MySystem {
    fn run(&mut self, ctx: &mut Context);
}

trait IntoSystem<Params> {
    type System: MySystem;
    fn into_system(self) -> Self::System;
}

struct FunctionSystem<F, Params: SystemParam> {
    system: F,
    _phantom: core::marker::PhantomData<Params>,
}

impl<F, Params: SystemParam + 'static> IntoSystem<Params> for F
where
    F: SystemParamFunction<Params> + 'static,
{
    type System = FunctionSystem<F, Params>;
    fn into_system(self) -> Self::System {
        FunctionSystem {
            system: self,
            _phantom: core::marker::PhantomData,
        }
    }
}

trait SystemParamFunction<Params: SystemParam>: 'static {
    fn run(&mut self, ctx: &mut Context);
}

impl<F, Params: SystemParam> MySystem for FunctionSystem<F, Params>
where
    F: SystemParamFunction<Params> + 'static,
{
    fn run(&mut self, ctx: &mut Context) {
        SystemParamFunction::run(&mut self.system, ctx)
    }
}

impl SystemParam for () {}
impl<A: SystemParam> SystemParam for (A,) {}
impl<A: SystemParam, B: SystemParam> SystemParam for (A, B) {}
impl<A: SystemParam, B: SystemParam, C: SystemParam> SystemParam for (A, B, C) {}
