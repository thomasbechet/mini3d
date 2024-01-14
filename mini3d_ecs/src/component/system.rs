use core::cell::RefCell;

use crate::error::SystemError;
use crate::view::SystemView;
use alloc::boxed::Box;
use mini3d_derive::Serialize;
use mini3d_utils::{slotmap::Key, string::AsciiArray};

use crate::{
    context::{Command, Context},
    entity::Entity,
    error::ComponentError,
    system::{ExclusiveSystem, ParallelSystem, SystemKey},
};

use super::{Component, ComponentStorage, EntityResolver};

pub struct NativeExclusiveSystem<Params, F> {
    function: F,
    params: Params,
}

pub struct NativeParallelSystem<Params, F> {
    function: F,
    params: Params,
}

macro_rules! impl_system {
    (
        $(
            $($params:ident),+
        )?
    ) => {

        #[allow(non_snake_case, unused)]
        impl<
            F: Fn(
                &mut Context,
                $($($params),+)?
            )
            $(,$($params: SystemView),+)?
        > ExclusiveSystem for NativeExclusiveSystem<($( $($params,)+ )?), F> {
            fn configure(&mut self) -> Result<(), SystemError> {
                // let ($($($params,)+)?) = &mut self.params;
                // (self.resolve)(config, $( $(($params),)+ )?)
                Ok(())
            }
            fn run(&mut self, ctx: &mut Context) -> Result<(), SystemError> {
                let ($($($params,)+)?) = &mut self.params;
                (self.function)(ctx, $( $(($params.clone()),)+ )?);
                Ok(())
            }
        }

        #[allow(non_snake_case, unused)]
        impl<
            F: Fn(
                &Context,
                $($($params),+)?
            )
            $(,$($params: SystemView),+)?
        > ParallelSystem for NativeParallelSystem<($( $($params,)+ )?), F> {
            fn configure(&mut self) -> Result<(), SystemError> {
                // let ($($($params,)+)?) = &mut self.params;
                // (self.resolve)(config, $( $(($params),)+ )?)
                Ok(())
            }
            fn run(&mut self, ctx: &Context) -> Result<(), SystemError> {
                let ($($($params,)+)?) = &mut self.params;
                (self.function)(ctx, $( $(($params.clone()),)+ )?);
                Ok(())
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

pub trait IntoNativeExclusiveSystem<Params> {
    type System: ExclusiveSystem + 'static;
    fn into_system(self) -> Self::System;
}

pub trait IntoNativeParallelSystem<Params> {
    type System: ParallelSystem + 'static;
    fn into_system(self) -> Self::System;
}

macro_rules! impl_into_system {
    (
        $($(
            $params:ident
        ),+)?
    ) => {
        impl<F: Fn(&mut Context, $($($params),+)?) + 'static $(, $($params: SystemView + 'static),+ )?> IntoNativeExclusiveSystem<( $($($params,)+)? )> for F {
            type System = NativeExclusiveSystem<( $($($params,)+)? ), F>;

            fn into_system(self) -> Self::System {
                NativeExclusiveSystem {
                    function: self,
                    params: Default::default(),
                }
            }
        }

        impl<F: Fn(&Context, $($($params),+)?) + 'static $(, $($params: SystemView + 'static),+ )?> IntoNativeParallelSystem<( $($($params,)+)? )> for F {
            type System = NativeParallelSystem<( $($($params,)+)? ), F>;

            fn into_system(self) -> Self::System {
                NativeParallelSystem {
                    function: self,
                    params: Default::default(),
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

#[derive(Default, Serialize)]
pub struct SystemOrder {}

#[derive(Default, Serialize)]
pub(crate) enum SystemKind {
    NativeExclusive(#[serialize(skip)] RefCell<Box<dyn ExclusiveSystem>>),
    NativeParallel(#[serialize(skip)] RefCell<Box<dyn ParallelSystem>>),
    #[default]
    Script,
}

impl Default for Box<dyn ExclusiveSystem> {
    fn default() -> Self {
        panic!("Invalid deserialization for native exclusive system")
    }
}

impl Default for Box<dyn ParallelSystem> {
    fn default() -> Self {
        panic!("Invalid deserialization for native parallel system")
    }
}

#[derive(Default, Serialize)]
pub struct System {
    pub(crate) name: AsciiArray<32>,
    pub(crate) kind: SystemKind,
    pub(crate) stage: Entity,
    pub(crate) order: SystemOrder,
    pub(crate) auto_enable: bool,
    #[serialize(skip)]
    pub(crate) key: SystemKey,
}

impl System {
    pub fn native_exclusive<Params>(
        name: &str,
        system: impl IntoNativeExclusiveSystem<Params>,
        stage: Entity,
        order: SystemOrder,
    ) -> Self {
        Self {
            name: AsciiArray::from(name),
            kind: SystemKind::NativeExclusive(RefCell::new(Box::new(system.into_system()))),
            stage,
            order,
            auto_enable: false,
            key: SystemKey::null(),
        }
    }

    pub fn native_parallel<Params>(
        name: &str,
        system: impl IntoNativeParallelSystem<Params>,
        stage: Entity,
        order: SystemOrder,
    ) -> Self {
        Self {
            name: AsciiArray::from(name),
            kind: SystemKind::NativeParallel(RefCell::new(Box::new(system.into_system()))),
            stage,
            order,
            auto_enable: false,
            key: SystemKey::null(),
        }
    }

    pub fn is_enable(&self) -> bool {
        !self.key.is_null()
    }
}

impl Component for System {
    const NAME: &'static str = "system";
    const STORAGE: ComponentStorage = ComponentStorage::Single;

    fn resolve_entities(&mut self, _: &mut EntityResolver) -> Result<(), ComponentError> {
        Ok(())
    }

    fn on_added(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if self.auto_enable {
            Command::enable_system(ctx, entity);
        }
        Ok(())
    }

    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if !self.key.is_null() {
            Command::disable_system(ctx, entity);
        }
        Ok(())
    }
}
