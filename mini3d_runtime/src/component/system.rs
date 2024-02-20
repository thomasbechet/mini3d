use alloc::{boxed::Box, vec::Vec};
use mini3d_derive::{Reflect, Serialize};

use crate::{
    ecs::{
        context::Context,
        entity::Entity,
        error::ResolverError,
        system::{SystemInstance, SystemKey},
        ECSCommand,
    },
    utils::{slotmap::Key, string::AsciiArray},
};

use super::{Component, ComponentError, ComponentKey, ComponentStorage};

pub trait NativeSystemParam: Default + Clone + 'static {}

pub trait NativeSystem {
    fn create_instance(&self) -> SystemInstance;
}

impl Default for Box<dyn NativeSystem> {
    fn default() -> Self {
        panic!("Invalid deserialize for native system")
    }
}

pub trait NativeExclusiveSystem {
    fn resolve(&mut self, config: &SystemConfig) -> Result<(), ResolverError>;
    fn run(&mut self, ctx: &mut Context);
}

pub trait NativeParallelSystem {
    fn resolve(&mut self, config: &SystemConfig) -> Result<(), ResolverError>;
    fn run(&mut self, ctx: &Context);
}

struct NativeExclusiveFunctionSystem<Params, F> {
    function: F,
    _marker: core::marker::PhantomData<Params>,
}

struct NativeParallelFunctionSystem<Params, F> {
    function: F,
    _marker: core::marker::PhantomData<Params>,
}

macro_rules! impl_system {
    (
        $(
            $($params:ident),+
        )?
    ) => {
        impl<
            F: Fn(
                &mut Context,
                $($($params),+)?
            )
            $(,$($params: NativeSystemParam),+)?
        > NativeSystem for NativeExclusiveFunctionSystem<($( $($params,)+ )?), F> {
            fn create_instance(&self) -> SystemInstance {
                struct Instance<Params: Default + 'static, F> {
                    function: F,
                    params: Params,
                }
                #[allow(non_snake_case, unused)]
                impl<
                    F: Fn(
                        &mut Context,
                        $($($params),+)?
                    )
                    $(,$($params: NativeSystemParam),+)?
                > NativeExclusiveSystem for Instance<($( $($params,)+ )?), F> {
                    fn run(&mut self, ctx: &mut Context) {
                        let ($($($params,)+)?) = &mut self.params;
                        (self.function)(ctx, $( $(($params.clone()),)+ )?);
                    }
                    fn resolve(&mut self, config: &SystemConfig) -> Result<(), ResolverError> {
                        // let ($($($params,)+)?) = &mut self.params;
                        // (self.resolve)(config, $( $(($params),)+ )?)
                        Ok(())
                    }
                }
                SystemInstance::NativeExclusive(Box::new(Instance {
                    function: self.function,
                    params: Default::default(),
                }))
            }
        }

        impl<
            F: Fn(
                &Context,
                $($($params),+)?
            )
            $(,$($params: NativeSystemParam),+)?
        > NativeSystem for NativeParallelFunctionSystem<($( $($params,)+ )?), F> {
            fn create_instance(&self) -> SystemInstance {
                struct Instance<Params: Default + 'static, F> {
                    function: F,
                    params: Params,
                }
                #[allow(non_snake_case, unused)]
                impl<
                    F: Fn(
                        &Context,
                        $($($params),+)?
                    )
                    $(,$($params: NativeSystemParam),+)?
                > NativeParallelSystem for Instance<($( $($params,)+ )?), F> {
                    fn run(&mut self, ctx: &Context) {
                        let ($($($params,)+)?) = &mut self.params;
                        (self.function)(ctx, $( $(($params.clone()),)+ )?);
                    }
                    fn resolve(&mut self, config: &SystemConfig) -> Result<(), ResolverError> {
                        // let ($($($params,)+)?) = &mut self.params;
                        // (self.resolve)(config, $( $(($params),)+ )?)
                        Ok(())
                    }
                }
                SystemInstance::NativeParallel(Box::new(Instance {
                    function: self.function,
                    params: Default::default(),
                }))
            }
        }
    }
}

impl_system!();
impl_system!(T1);
impl_system!(T1, T2);
impl_system!(T1, T2, T3);
impl_system!(T1, T2, T3, T4);
impl_system!(T1, T2, T3, T4, T5);
impl_system!(T1, T2, T3, T4, T5, T6);
impl_system!(T1, T2, T3, T4, T5, T6, T7);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8);

trait IntoNativeExclusiveSystem<Params> {
    type System: NativeSystem + 'static;
    fn into_system(self) -> Self::System;
}

trait IntoNativeParallelSystem<Params> {
    type System: NativeSystem + 'static;
    fn into_system(self) -> Self::System;
}

macro_rules! impl_into_system {
    (
        $($(
            $params:ident
        ),+)?
    ) => {
        impl<F: Fn(&mut Context, $($($params),+)?) + 'static $(, $($params: NativeSystemParam),+ )?> IntoNativeExclusiveSystem<( $($($params,)+)? )> for F {
            type System = NativeExclusiveFunctionSystem<( $($($params,)+)? ), F>;

            fn into_system(self) -> Self::System {
                NativeExclusiveFunctionSystem {
                    function: self,
                    _marker: Default::default(),
                }
            }
        }

        impl<F: Fn(&Context, $($($params),+)?) + 'static $(, $($params: NativeSystemParam),+ )?> IntoNativeParallelSystem<( $($($params,)+)? )> for F {
            type System = NativeParallelFunctionSystem<( $($($params,)+)? ), F>;

            fn into_system(self) -> Self::System {
                NativeParallelFunctionSystem {
                    function: self,
                    _marker: Default::default(),
                }
            }
        }
    }
}

impl_into_system!();
impl_into_system!(T1);
impl_into_system!(T1, T2);
impl_into_system!(T1, T2, T3);
impl_into_system!(T1, T2, T3, T4);
impl_into_system!(T1, T2, T3, T4, T5);
impl_into_system!(T1, T2, T3, T4, T5, T6);
impl_into_system!(T1, T2, T3, T4, T5, T6, T7);
impl_into_system!(T1, T2, T3, T4, T5, T6, T7, T8);

pub enum SystemParam<'a> {
    ViewRef(&'a str),
    ViewMut(&'a str),
    Query {
        all: &'a [&'a str],
        any: &'a [&'a str],
        not: &'a [&'a str],
    },
}

enum SystemParamEntry {
    ViewRef(ComponentKey),
    ViewMut(ComponentKey),
    Query,
    QueryAll(u16),
    QueryAny(u16),
    QueryNot(u16),
}

#[derive(Default)]
pub struct SystemConfig {
    params: Vec<SystemParamEntry>,
}

impl SystemConfig {
    pub const fn new(params: &[SystemParam]) -> Self {
        Self { params: Vec::new() }
    }
}

#[derive(Serialize)]
pub(crate) enum SystemKind {
    Native(#[serialize(skip)] Box<dyn NativeSystem>),
    Script,
}

impl Default for SystemKind {
    fn default() -> Self {
        Self::Script
    }
}

#[derive(Default, Serialize, Reflect)]
pub struct SystemOrder {}

#[derive(Default, Serialize, Reflect)]
pub struct System {
    pub(crate) name: AsciiArray<32>,
    pub(crate) kind: SystemKind,
    pub(crate) stage: Entity,
    #[serialize(skip)]
    pub(crate) config: SystemConfig,
    pub(crate) order: SystemOrder,
    #[serialize(skip)]
    pub(crate) key: SystemKey,
}

impl System {
    pub const NAME: &'static str = "system";

    pub fn native_exclusive<Params>(
        name: &str,
        system: impl IntoNativeExclusiveSystem<Params>,
        stage: Entity,
        config: SystemConfig,
        order: SystemOrder,
    ) -> Self {
        Self {
            name: AsciiArray::from(name),
            kind: SystemKind::Native(Box::new(system.into_system())),
            stage,
            config,
            order,
            key: SystemKey::null(),
        }
    }

    pub fn native_parallel<Params>(
        name: &str,
        system: impl IntoNativeParallelSystem<Params>,
        stage: Entity,
        config: SystemConfig,
        order: SystemOrder,
    ) -> Self {
        Self {
            name: AsciiArray::from(name),
            kind: SystemKind::Native(Box::new(system.into_system())),
            stage,
            config,
            order,
            key: SystemKey::null(),
        }
    }

    pub fn script(name: &str) -> Self {
        Self {
            name: AsciiArray::from(name),
            kind: SystemKind::Script,
            stage: Default::default(),
            config: Default::default(),
            order: Default::default(),
            key: SystemKey::null(),
        }
    }

    pub fn enable(&mut self, ctx: &mut Context, enable: bool) {}
}

impl Component for System {
    const STORAGE: ComponentStorage = ComponentStorage::Single;

    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        ctx.ecs.commands.push(ECSCommand::AddSystem(entity));
        Ok(())
    }

    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        ctx.ecs.commands.push(ECSCommand::RemoveSystem(entity));
        Ok(())
    }
}
