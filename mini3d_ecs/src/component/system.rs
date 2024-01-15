use core::cell::RefCell;

use crate::system::GlobalSystem;
use crate::view::SystemView;
use crate::{error::SystemError, world::World};
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::sync::Arc;
use mini3d_derive::Serialize;
use mini3d_utils::slotmap::Key;

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

pub struct NativeGlobalSystem<F> {
    function: F,
}

impl<F> GlobalSystem for NativeGlobalSystem<F>
where
    F: Fn(&mut Context, &mut World) + 'static,
{
    fn run(&mut self, ctx: &mut Context, world: &mut World) -> Result<(), SystemError> {
        (self.function)(ctx, world);
        Ok(())
    }
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
            ) + 'static
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
            ) + 'static
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
    NativeExclusive(#[serialize(skip)] Rc<RefCell<Box<dyn ExclusiveSystem>>>),
    NativeParallel(#[serialize(skip)] Arc<RefCell<Box<dyn ParallelSystem>>>),
    NativeGlobal(#[serialize(skip)] Rc<RefCell<Box<dyn GlobalSystem>>>),
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

impl Default for Box<dyn GlobalSystem> {
    fn default() -> Self {
        panic!("Invalid deserialization for native global system")
    }
}

#[derive(Default, Serialize)]
pub struct System {
    pub(crate) kind: SystemKind,
    pub(crate) stage: Entity,
    pub(crate) order: SystemOrder,
    pub(crate) auto_enable: bool,
    #[serialize(skip)]
    pub(crate) key: SystemKey,
}

impl System {
    pub const IDENT: &'static str = "system";

    pub fn native_exclusive<Params>(
        function: impl IntoNativeExclusiveSystem<Params>,
        stage: Entity,
        order: SystemOrder,
    ) -> Self {
        Self {
            kind: SystemKind::NativeExclusive(Rc::new(RefCell::new(Box::new(
                function.into_system(),
            )))),
            stage,
            order,
            auto_enable: false,
            key: SystemKey::null(),
        }
    }

    pub fn native_parallel<Params>(
        function: impl IntoNativeParallelSystem<Params>,
        stage: Entity,
        order: SystemOrder,
    ) -> Self {
        Self {
            kind: SystemKind::NativeParallel(Arc::new(RefCell::new(Box::new(
                function.into_system(),
            )))),
            stage,
            order,
            auto_enable: false,
            key: SystemKey::null(),
        }
    }

    pub fn native_global(
        function: fn(&mut Context, &mut World),
        stage: Entity,
        order: SystemOrder,
    ) -> Self {
        Self {
            kind: SystemKind::NativeGlobal(Rc::new(RefCell::new(Box::new(NativeGlobalSystem {
                function,
            })))),
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
