use core::cell::RefCell;

use crate::container::ContainerTable;
use crate::error::SystemError;
use crate::instance::Instance;
use crate::instance::InstanceIndex;
use crate::instance::InstanceTable;
use crate::instance::SystemResolver;
use crate::view::SystemView;
use alloc::boxed::Box;
use alloc::rc::Rc;
use alloc::sync::Arc;
use mini3d_derive::Serialize;

use crate::{
    context::Context,
    entity::Entity,
    error::ComponentError,
    instance::{ExclusiveSystem, ParallelSystem},
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
            ) + 'static
            $(,$($params: SystemView),+)?
        > ExclusiveSystem for NativeExclusiveSystem<($( $($params,)+ )?), F> {
            fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), SystemError> {
                let ($($($params,)+)?) = &mut self.params;
                $($($params.resolve(resolver)?;)+)?
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
            fn resolve(&mut self, resolver: &mut SystemResolver) -> Result<(), SystemError> {
                let ($($($params,)+)?) = &mut self.params;
                $($($params.resolve(resolver)?;)+)?
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
    NativeExclusive {
        #[serialize(skip)]
        system: Rc<RefCell<Box<dyn ExclusiveSystem>>>,
        views: [Entity; 8],
    },
    NativeParallel {
        #[serialize(skip)]
        system: Arc<RefCell<Box<dyn ParallelSystem>>>,
        views: [Entity; 8],
    },
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
    pub(crate) kind: SystemKind,
    pub(crate) stage: Entity,
    pub(crate) order: SystemOrder,
    pub(crate) auto_enable: bool,
    pub(crate) instance: Option<InstanceIndex>,
}

impl System {
    pub fn exclusive<Params>(
        function: impl IntoNativeExclusiveSystem<Params>,
        stage: Entity,
        order: SystemOrder,
        dynamic_views: &[Entity],
    ) -> Self {
        Self {
            kind: SystemKind::NativeExclusive {
                system: Rc::new(RefCell::new(Box::new(function.into_system()))),
                views: [
                    dynamic_views.get(0).copied().unwrap_or_default(),
                    dynamic_views.get(1).copied().unwrap_or_default(),
                    dynamic_views.get(2).copied().unwrap_or_default(),
                    dynamic_views.get(3).copied().unwrap_or_default(),
                    dynamic_views.get(4).copied().unwrap_or_default(),
                    dynamic_views.get(5).copied().unwrap_or_default(),
                    dynamic_views.get(6).copied().unwrap_or_default(),
                    dynamic_views.get(7).copied().unwrap_or_default(),
                ],
            },
            stage,
            order,
            auto_enable: false,
            instance: None,
        }
    }

    pub fn parallel<Params>(
        function: impl IntoNativeParallelSystem<Params>,
        stage: Entity,
        order: SystemOrder,
        dynamic_views: &[Entity],
    ) -> Self {
        Self {
            kind: SystemKind::NativeParallel {
                system: Arc::new(RefCell::new(Box::new(function.into_system()))),
                views: [
                    dynamic_views.get(0).copied().unwrap_or_default(),
                    dynamic_views.get(1).copied().unwrap_or_default(),
                    dynamic_views.get(2).copied().unwrap_or_default(),
                    dynamic_views.get(3).copied().unwrap_or_default(),
                    dynamic_views.get(4).copied().unwrap_or_default(),
                    dynamic_views.get(5).copied().unwrap_or_default(),
                    dynamic_views.get(6).copied().unwrap_or_default(),
                    dynamic_views.get(7).copied().unwrap_or_default(),
                ],
            },
            stage,
            order,
            auto_enable: false,
            instance: None,
        }
    }

    pub fn is_active(&self) -> bool {
        self.instance.is_some()
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
            Self::enable(ctx, entity);
        }
        Ok(())
    }

    fn on_removed(&mut self, entity: Entity, ctx: &mut Context) -> Result<(), ComponentError> {
        if self.is_active() {
            Self::disable(ctx, entity);
        }
        Ok(())
    }
}

pub(crate) fn enable_system(
    entity: Entity,
    instances: &mut InstanceTable,
    containers: &mut ContainerTable,
) -> Result<(), ComponentError> {
    let systems = containers.systems();
    let system = systems
        .get_mut(entity)
        .ok_or(ComponentError::EntryNotFound)?;
    let instance = match system.kind {
        SystemKind::NativeExclusive { ref mut system, .. } => Instance::Exclusive(system.clone()),
        SystemKind::NativeParallel { ref mut system, .. } => Instance::Parallel(system.clone()),
        SystemKind::Script => {
            return Err(ComponentError::UnresolvedReference);
        }
    };
    instances.entries.push(instance);
    system.instance = Some(InstanceIndex(instances.entries.len() as u16 - 1));
    // Resolve system, TODO: handle views and proper initialization
    instances.entries[system.instance.unwrap().0 as usize]
        .resolve(&mut SystemResolver {
            containers,
            views: &[],
            index: 0,
        })
        .map_err(|_| ComponentError::UnresolvedReference)?;
    Ok(())
}

pub(crate) fn disable_system(
    entity: Entity,
    instances: &mut InstanceTable,
    containers: &mut ContainerTable,
) {
    unimplemented!()
}
