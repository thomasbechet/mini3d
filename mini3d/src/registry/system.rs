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
    fn from(id: System) -> Self {
        id.0
    }
}

pub(crate) enum SystemStageKind<'a> {
    Update,
    FixedUpdate(f64),
    Custom(&'a str),
}

pub struct SystemStage<'a> {
    kind: SystemStageKind<'a>,
}

impl<'a> SystemStage<'a> {
    pub const UPDATE: SystemStage<'static> = Self {
        kind: SystemStageKind::Update,
    };
    pub const FIXED_UPDATE_60HZ: SystemStage<'static> = Self::fixed_update(60.0);
    pub const SCENE_CHANGED: SystemStage<'static> = Self::custom("scene_changed");
    pub const SCENE_START: SystemStage<'static> = Self::custom("scene_start");
    pub const SCENE_STOP: SystemStage<'static> = Self::custom("scene_stop");

    pub const fn fixed_update(frequency: f64) -> Self {
        Self {
            kind: SystemStageKind::FixedUpdate(frequency),
        }
    }

    pub const fn custom(event: &'a str) -> Self {
        Self {
            kind: SystemStageKind::Custom(event),
        }
    }
}

pub(crate) const MAX_SYSTEM_NAME_LEN: usize = 64;
pub(crate) const MAX_SYSTEM_STAGE_NAME_LEN: usize = 64;

pub(crate) struct SystemStageDefinition {
    pub(crate) name: AsciiArray<MAX_SYSTEM_STAGE_NAME_LEN>,
    uid: UID,
    ref_count: usize,
}

pub(crate) struct SystemDefinition {
    pub(crate) name: AsciiArray<MAX_SYSTEM_NAME_LEN>,
    uid: UID,
    pub(crate) reflection: Box<dyn AnySystemReflection>,
    stage: SlotId,
}

#[derive(Default)]
pub(crate) struct SystemRegistry {
    systems: SlotMap<SystemDefinition>,
    stages: SlotMap<SystemStageDefinition>,
}

impl SystemRegistry {
    fn add_system(&mut self, definition: SystemDefinition) -> Result<System, RegistryError> {
        if self.find(definition.uid).is_some() {
            return Err(RegistryError::DuplicatedSystemDefinition {
                name: definition.name.to_string(),
            });
        }
        let id = self.systems.add(definition);
        self.stages[definition.stage].ref_count += 1;
        Ok(id.into())
    }

    fn get_or_add_system_stage(&mut self, stage: SystemStage) -> SlotId {
        for (id, stage) in self.stages.iter() {
            if stage.uid == stage {
                return id;
            }
        }
        self.stages.add(SystemStageDefinition {
            name: stage.uid().into(),
            uid: stage.uid(),
            ref_count: 0,
        })
    }

    pub(crate) fn add_static_exclusive<S: ExclusiveSystem>(
        &mut self,
        name: &str,
        stage: SystemStage,
    ) -> Result<System, RegistryError> {
        let stage = self.get_or_add_system_stage(stage);
        self.add_system(SystemDefinition {
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
        let stage = self.get_or_add_system_stage(stage);
        self.add_system(SystemDefinition {
            name: name.into(),
            uid: S::UID,
            reflection: Box::new(StaticParallelSystemReflection::<S> {
                _phantom: std::marker::PhantomData,
            }),
            stage,
        })
    }

    pub(crate) fn find(&self, uid: UID) -> Option<&SystemDefinition> {
        self.systems.get(uid.into())
    }

    pub(crate) fn get(&self, system: System) -> Option<&SystemDefinition> {
        self.systems.get(system.into())
    }
}
