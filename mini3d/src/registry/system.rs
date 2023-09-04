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

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SystemStage(SlotId);

impl From<SlotId> for System {
    fn from(id: SlotId) -> Self {
        Self(id)
    }
}

impl From<System> for SlotId {
    fn from(id: System) -> Self {
        id.0
    }
}

pub(crate) enum SystemStageBuildKind<'a> {
    Update,
    FixedUpdate(f64),
    Custom(&'a str),
}

pub struct SystemStageDefinition<'a> {
    kind: SystemStageBuildKind<'a>,
}

impl SystemStage {
    pub const UPDATE: SystemStageDefinition<'static> = SystemStageDefinition {
        kind: SystemStageBuildKind::Update,
    };
    pub const FIXED_UPDATE_60HZ: SystemStageDefinition<'static> = Self::fixed_update(60.0);
    pub const SCENE_CHANGED: SystemStageDefinition<'static> = Self::custom("scene_changed");
    pub const SCENE_START: SystemStageDefinition<'static> = Self::custom("scene_start");
    pub const SCENE_STOP: SystemStageDefinition<'static> = Self::custom("scene_stop");

    pub const fn fixed_update(frequency: f64) -> Self {
        SystemStageDefinition {
            kind: SystemStageBuildKind::FixedUpdate(frequency),
        }
    }

    pub const fn custom(event: &str) -> Self {
        SystemStageDefinition {
            kind: SystemStageBuildKind::Custom(event),
        }
    }
}

pub(crate) const MAX_SYSTEM_NAME_LEN: usize = 64;
pub(crate) const MAX_SYSTEM_STAGE_NAME_LEN: usize = 64;

pub(crate) enum SystemStageKind {
    Update,
    FixedUpdate(f64),
    Custom,
}

pub(crate) struct SystemStageEntry {
    pub(crate) name: AsciiArray<MAX_SYSTEM_STAGE_NAME_LEN>,
    pub(crate) uid: UID,
    pub(crate) kind: SystemStageKind,
    ref_count: usize,
}

pub(crate) struct SystemEntry {
    pub(crate) name: AsciiArray<MAX_SYSTEM_NAME_LEN>,
    pub(crate) uid: UID,
    pub(crate) reflection: Box<dyn AnySystemReflection>,
    pub(crate) stage: SlotId,
}

#[derive(Default)]
pub(crate) struct SystemRegistry {
    systems: SlotMap<SystemEntry>,
    stages: SlotMap<SystemStageEntry>,
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

    fn get_or_add_system_stage(&mut self, stage: SystemStage) -> Result<SlotId, RegistryError> {
        let kind = match stage.kind {
            SystemStageBuildKind::Update => SystemStageKind::Update,
            SystemStageBuildKind::FixedUpdate(frequency) => SystemStageKind::FixedUpdate(frequency),
            SystemStageBuildKind::Custom(_) => SystemStageKind::Custom,
        };
        let name = AsciiArray::from(match stage.kind {
            SystemStageBuildKind::Update => "update",
            SystemStageBuildKind::FixedUpdate(frequency) => {
                format!("fixed_update_{:.2}hz", frequency).as_str()
            }
            SystemStageBuildKind::Custom(name) => name,
        });
        let uid = UID::from(name);
        for (id, def) in self.stages.iter() {
            if def.uid == uid {
                if !matches!(def.kind, kind) {
                    return Err(RegistryError::IncompatibleSystemStageDefinition);
                }
                return Ok(id);
            }
        }
        Ok(self.stages.add(SystemStageEntry {
            name,
            uid,
            kind,
            ref_count: 0,
        }))
    }

    pub(crate) fn add_static_exclusive<S: ExclusiveSystem>(
        &mut self,
        name: &str,
        stage: SystemStage,
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
        stage: SystemStage,
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

    pub(crate) fn find(&self, uid: UID) -> Option<System> {
        self.systems
            .iter()
            .find(|(_, def)| def.uid == uid)
            .map(|(id, _)| id.into())
    }

    pub(crate) fn find_stage(&self, uid: UID) -> Option<SystemStage> {
        self.stages
            .iter()
            .find(|(_, def)| def.uid == uid)
            .map(|(id, _)| id.into())
    }

    pub(crate) fn get(&self, system: System) -> Option<&SystemEntry> {
        self.systems.get(system.into())
    }

    pub(crate) fn get_stage(&self, stage: SystemStage) -> Option<&SystemStageEntry> {
        self.stages.get(stage.into())
    }
}
