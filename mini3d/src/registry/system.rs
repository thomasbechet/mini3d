use crate::{
    ecs::{
        api::{
            ecs::{ExclusiveECS, ParallelECS},
            ExclusiveAPI, ParallelAPI,
        },
        instance::{
            AnyStaticExclusiveSystemInstance, AnyStaticParallelSystemInstance, ExclusiveResolver,
            ExclusiveSystemInstance, ParallelResolver, ParallelSystemInstance, SystemInstance,
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
    fn setup(&mut self, resolver: &mut ExclusiveResolver) -> Result<(), RegistryError> {
        Ok(())
    }
    fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) {}
}

pub trait ParallelSystem: 'static + Default {
    fn setup(&mut self, resolver: &mut ParallelResolver) -> Result<(), RegistryError> {
        Ok(())
    }
    fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) {}
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
            fn run(&self, ecs: &mut ExclusiveECS, api: &mut ExclusiveAPI) {
                self.system.run(ecs, api);
            }
        }
        SystemInstance::Exclusive(ExclusiveSystemInstance::Static(Box::new(InstanceHolder {
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
            fn run(&self, ecs: &mut ParallelECS, api: &mut ParallelAPI) {
                self.system.run(ecs, api);
            }
        }
        SystemInstance::Parallel(ParallelSystemInstance::Static(Box::new(InstanceHolder {
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
}

pub const MAX_SYSTEM_NAME_LEN: usize = 64;
pub const MAX_SYSTEM_STAGE_NAME_LEN: usize = 64;

pub(crate) struct SystemStageEntry {
    pub(crate) name: AsciiArray<MAX_SYSTEM_STAGE_NAME_LEN>,
    pub(crate) uid: UID,
    pub(crate) first_system: Option<System>,
    core_stage: bool, // Ensure this stage is not removed
}

pub(crate) struct SystemEntry {
    pub(crate) name: AsciiArray<MAX_SYSTEM_NAME_LEN>,
    pub(crate) uid: UID,
    pub(crate) reflection: Box<dyn AnySystemReflection>,
    pub(crate) stage: SlotId,
    pub(crate) next_in_stage: Option<System>,
    pub(crate) prev_in_stage: Option<System>,
    pub(crate) active_by_default: bool,
}

#[derive(Default)]
pub struct SystemOrder;

pub(crate) struct SystemRegistry {
    pub(crate) systems: SlotMap<SystemEntry>,
    pub(crate) stages: SlotMap<SystemStageEntry>,
}

impl Default for SystemRegistry {
    fn default() -> Self {
        let mut reg = Self {
            systems: Default::default(),
            stages: Default::default(),
        };
        for name in [SystemStage::UPDATE, SystemStage::FIXED_UPDATE_60HZ] {
            reg.stages.add(SystemStageEntry {
                name: AsciiArray::from(name),
                uid: UID::new(name),
                first_system: None,
                core_stage: true,
            });
        }
        reg
    }
}

impl SystemRegistry {
    pub(crate) fn log(&self) {
        println!("=== SYSTEMS ===");
        for (id, entry) in self.systems.iter() {
            println!("- {} {:?}", entry.name.as_str(), id);
        }
        println!("=== STAGES ===");
        for (_, entry) in self.stages.iter() {
            println!("- {}", entry.name.as_str());
        }
    }

    fn find_stage(&self, stage: UID) -> Option<SlotId> {
        self.stages
            .iter()
            .find(|(_, entry)| UID::new(&entry.name) == stage)
            .map(|(id, _)| id)
    }

    fn find_last_system_in_stage(&self, stage: SlotId) -> Option<System> {
        let mut system = self.stages[stage].first_system;
        while let Some(id) = system {
            let entry = &self.systems[id.into()];
            if entry.next_in_stage.is_none() {
                return Some(id);
            }
            system = entry.next_in_stage;
        }
        None
    }

    fn add_system(&mut self, entry: SystemEntry) -> Result<System, RegistryError> {
        let stage = entry.stage;
        let id = self.systems.add(entry);
        if let Some(last) = self.find_last_system_in_stage(stage) {
            let last = last.into();
            self.systems[last].next_in_stage = Some(id.into());
            self.systems[id].prev_in_stage = Some(last.into());
        } else {
            self.stages[stage].first_system = Some(id.into());
        }
        Ok(id.into())
    }

    pub(crate) fn remove(&mut self, system: System) {
        let system = system.into();
        let stage = self.systems[system].stage;
        if let Some(prev) = self.systems[system].prev_in_stage {
            self.systems[prev.into()].next_in_stage = self.systems[system].next_in_stage;
        } else {
            self.stages[stage].first_system = self.systems[system].next_in_stage;
        }
        if self.stages[stage].first_system.is_none() && !self.stages[stage].core_stage {
            self.stages.remove(stage);
        }
        self.systems.remove(system);
    }

    fn get_or_add_system_stage(&mut self, name: &str) -> SlotId {
        let uid = UID::from(name);
        for (id, def) in self.stages.iter() {
            if def.uid == uid {
                return id;
            }
        }
        self.stages.add(SystemStageEntry {
            name: AsciiArray::from(name),
            uid,
            first_system: None,
            core_stage: false,
        })
    }

    pub(crate) fn add_static_exclusive<S: ExclusiveSystem>(
        &mut self,
        name: &str,
        stage: &str,
        order: SystemOrder,
    ) -> Result<System, RegistryError> {
        let stage = self.get_or_add_system_stage(stage);
        self.add_system(SystemEntry {
            name: name.into(),
            uid: name.into(),
            reflection: Box::new(StaticExclusiveSystemReflection::<S> {
                _phantom: std::marker::PhantomData,
            }),
            stage,
            active_by_default: true,
            next_in_stage: None,
            prev_in_stage: None,
        })
    }

    pub(crate) fn add_static_parallel<S: ParallelSystem>(
        &mut self,
        name: &str,
        stage: &str,
        order: SystemOrder,
    ) -> Result<System, RegistryError> {
        let stage = self.get_or_add_system_stage(stage);
        self.add_system(SystemEntry {
            name: name.into(),
            uid: name.into(),
            reflection: Box::new(StaticParallelSystemReflection::<S> {
                _phantom: std::marker::PhantomData,
            }),
            stage,
            active_by_default: true,
            next_in_stage: None,
            prev_in_stage: None,
        })
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
