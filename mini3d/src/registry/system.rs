use std::collections::HashMap;

use crate::{
    ecs::{
        context::{ExclusiveContext, ParallelContext},
        system::{
            AnyStaticExclusiveSystemInstance, AnyStaticParallelSystemInstance, ExclusiveResolver,
            ParallelResolver, StaticSystemInstance, SystemInstance, SystemResult,
        },
    },
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
        uid::UID,
    },
};

use super::error::RegistryError;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct SystemId(SlotId);

impl From<SlotId> for SystemId {
    fn from(id: SlotId) -> Self {
        Self(id)
    }
}

impl From<SystemId> for SlotId {
    fn from(id: SystemId) -> Self {
        id.0
    }
}

pub trait ExclusiveSystem: 'static + Default {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult;
}

pub trait ParallelSystem: 'static + Default {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn resolve(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError>;
    fn run(&self, ctx: &mut ParallelContext) -> SystemResult;
}

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
            fn resolve(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
                self.system.resolve(resolver)
            }
            fn run(&self, ctx: &mut ExclusiveContext) -> SystemResult {
                self.system.run(ctx)
            }
        }
        SystemInstance::Static(StaticSystemInstance::Exclusive(Box::new(InstanceHolder {
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
            ) -> Result<(), RegistryError> {
                self.system.resolve(resolver)
            }
            fn run(&self, ctx: &mut ParallelContext) -> SystemResult {
                self.system.run(ctx)
            }
        }
        SystemInstance::Static(StaticSystemInstance::Parallel(Box::new(InstanceHolder {
            system: S::default(),
        })))
    }
}

pub(crate) const MAX_SYSTEM_NAME_LEN: usize = 64;

pub(crate) struct SystemDefinition {
    pub(crate) name: AsciiArray<MAX_SYSTEM_NAME_LEN>,
    pub(crate) reflection: Box<dyn AnySystemReflection>,
}

#[derive(Default)]
pub(crate) struct SystemRegistry {
    systems: SlotMap<SystemDefinition>,
    lookup_cache: HashMap<UID, SystemId>,
}

impl SystemRegistry {
    fn define(&mut self, definition: SystemDefinition) -> Result<SystemId, RegistryError> {
        let uid: UID = definition.name.as_str().into();
        if self.find(uid).is_some() {
            return Err(RegistryError::DuplicatedSystemDefinition {
                name: definition.name.to_string(),
            });
        }
        let id = self.systems.add(definition);
        self.lookup_cache.insert(uid, id.into());
        Ok(id.into())
    }

    fn resolve_components(&mut self) -> Result<(), RegistryError> {
        // TODO: Resolve components
        Ok(())
    }

    pub(crate) fn define_static_exclusive<S: ExclusiveSystem>(
        &mut self,
        name: &str,
    ) -> Result<SystemId, RegistryError> {
        self.define(SystemDefinition {
            name: name.into(),
            reflection: Box::new(StaticExclusiveSystemReflection::<S> {
                _phantom: std::marker::PhantomData,
            }),
        })
    }

    pub(crate) fn define_static_parallel<S: ParallelSystem>(
        &mut self,
        name: &str,
    ) -> Result<SystemId, RegistryError> {
        self.define(SystemDefinition {
            name: name.into(),
            reflection: Box::new(StaticParallelSystemReflection::<S> {
                _phantom: std::marker::PhantomData,
            }),
        })
    }

    pub(crate) fn find(&self, uid: UID) -> Option<SystemId> {
        self.lookup_cache.get(&uid).copied()
    }

    pub(crate) fn get(&self, id: SystemId) -> Option<&SystemDefinition> {
        self.systems.get(id.into())
    }
}
