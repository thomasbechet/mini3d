use crate::{
    ecs::{
        api::{
            ecs::{ExclusiveECS, ParallelECS},
            ExclusiveAPI, ParallelAPI,
        },
        scheduler::{StaticSystemInstance, SystemInstance},
        system::{
            AnyStaticExclusiveSystemInstance, AnyStaticParallelSystemInstance, ExclusiveResolver,
            ParallelResolver, SystemResult,
        },
    },
    utils::{
        slotmap::{SlotId, SlotMap},
        string::AsciiArray,
        uid::UID,
    },
};

use super::error::RegistryError;

pub trait ExclusiveSystem: 'static + Default {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        Ok(())
    }
    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult {
        Ok(())
    }
}

pub trait ParallelSystem: 'static + Default {
    const NAME: &'static str;
    const UID: UID = UID::new(Self::NAME);
    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        Ok(())
    }
    fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) -> SystemResult {
        Ok(())
    }
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
                self.system.setup(resolver)
            }
            fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) -> SystemResult {
                self.system.run(ecs, api)
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
                self.system.setup(resolver)
            }
            fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) -> SystemResult {
                self.system.run(ecs, api)
            }
        }
        SystemInstance::Static(StaticSystemInstance::Parallel(Box::new(InstanceHolder {
            system: S::default(),
        })))
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct System(SlotId);

impl From<SlotId> for System {
    fn from(id: SlotId) -> Self {
        Self(id)
    }
}

impl From<System> for SlotId {
    fn from(system: System) -> Self {
        system.0
    }
}

pub struct SystemStage;

impl SystemStage {
    pub const UPDATE: &'static str = "update";
    pub const FIXED_UPDATE_60HZ: &'static str = "fixed_update_60hz";
    pub const SCENE_CHANGED: &'static str = "scene_changed";
    pub const SCENE_START: &'static str = "scene_start";
    pub const SCENE_STOP: &'static str = "scene_stop";
}

pub const MAX_SYSTEM_NAME_LEN: usize = 64;
pub const MAX_SYSTEM_STAGE_NAME_LEN: usize = 64;

pub(crate) struct SystemStageEntry {
    pub(crate) name: AsciiArray<MAX_SYSTEM_STAGE_NAME_LEN>,
    pub(crate) uid: UID,
    ref_count: usize,
}

pub(crate) struct SystemEntry {
    pub(crate) name: AsciiArray<MAX_SYSTEM_NAME_LEN>,
    pub(crate) uid: UID,
    pub(crate) reflection: Box<dyn AnySystemReflection>,
    pub(crate) stage: SlotId,
}

pub(crate) struct SystemRegistry {
    systems: SlotMap<SystemEntry>,
    stages: SlotMap<SystemStageEntry>,
}

impl Default for SystemRegistry {
    fn default() -> Self {
        let mut reg = Self {
            systems: Default::default(),
            stages: Default::default(),
        };
        for name in [
            SystemStage::UPDATE,
            SystemStage::FIXED_UPDATE_60HZ,
            SystemStage::SCENE_CHANGED,
            SystemStage::SCENE_START,
            SystemStage::SCENE_STOP,
        ] {
            reg.stages.add(SystemStageEntry {
                name: AsciiArray::from(name),
                uid: UID::new(name),
                ref_count: 0,
            });
        }
        reg
    }
}

impl SystemRegistry {
    fn add_system(&mut self, definition: SystemEntry) -> Result<System, RegistryError> {
        if self.find(definition.uid).is_some() {
            return Err(RegistryError::DuplicatedSystemDefinition {
                name: definition.name.to_string(),
            });
        }
        let id = self.systems.add(definition);
        self.stages[definition.stage].ref_count += 1;
        Ok(id.into())
    }

    fn get_or_add_system_stage(&mut self, name: &str) -> Result<SlotId, RegistryError> {
        let uid = UID::from(name);
        for (id, def) in self.stages.iter() {
            if def.uid == uid {
                return Ok(id);
            }
        }
        Ok(self.stages.add(SystemStageEntry {
            name: AsciiArray::from(name),
            uid,
            ref_count: 0,
        }))
    }

    pub(crate) fn add_static_exclusive<S: ExclusiveSystem>(
        &mut self,
        name: &str,
        stage: &str,
    ) -> Result<System, RegistryError> {
        let stage = self.get_or_add_system_stage(stage)?;
        self.add_system(SystemEntry {
            name: name.into(),
            uid: S::UID,
            reflection: Box::new(StaticExclusiveSystemReflection::<S> {
                _phantom: std::marker::PhantomData,
            }),
            stage,
        })
    }

    pub(crate) fn add_static_parallel<S: ParallelSystem>(
        &mut self,
        name: &str,
        stage: &str,
    ) -> Result<System, RegistryError> {
        let stage = self.get_or_add_system_stage(stage)?;
        self.add_system(SystemEntry {
            name: name.into(),
            uid: S::UID,
            reflection: Box::new(StaticParallelSystemReflection::<S> {
                _phantom: std::marker::PhantomData,
            }),
            stage,
        })
    }

    pub(crate) fn remove(&mut self, system: System) {
        todo!()
    }

    pub(crate) fn find(&self, uid: UID) -> Option<System> {
        self.systems
            .iter()
            .find(|(_, def)| def.uid == uid)
            .map(|(id, _)| id.into())
    }

    pub(crate) fn get(&self, system: System) -> Option<&SystemEntry> {
        self.systems.get(system.into())
    }
}
