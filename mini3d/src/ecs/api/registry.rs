use crate::{
    registry::{
        component::{ComponentData, ComponentHandle, ComponentRegistry},
        error::RegistryError,
        system::{ExclusiveSystem, ParallelSystem, System, SystemOrder, SystemRegistry},
    },
    utils::uid::UID,
};

pub struct ParallelSystemRegistryAPI<'a> {
    pub(crate) manager: &'a SystemRegistry,
}

pub struct ParallelComponentRegistryAPI<'a> {
    pub(crate) manager: &'a ComponentRegistry,
}

pub struct ParallelRegistryAPI<'a> {
    pub systems: ParallelSystemRegistryAPI<'a>,
    pub components: ParallelComponentRegistryAPI<'a>,
}

pub struct ExclusiveSystemRegistryAPI<'a> {
    pub(crate) manager: &'a mut SystemRegistry,
}

impl<'a> ExclusiveSystemRegistryAPI<'a> {
    pub fn add_static_exclusive<S: ExclusiveSystem>(
        &mut self,
        name: &str,
        stage: &str,
        order: SystemOrder,
    ) -> Result<System, RegistryError> {
        self.manager.add_static_exclusive::<S>(name, stage, order)
    }

    pub fn add_static_parallel<S: ParallelSystem>(
        &mut self,
        name: &str,
        stage: &str,
        order: SystemOrder,
    ) -> Result<System, RegistryError> {
        self.manager.add_static_parallel::<S>(name, stage, order)
    }

    pub fn remove(&mut self, system: System) {
        self.manager.remove(system)
    }

    pub fn find(&self, uid: UID) -> Option<System> {
        self.manager.find(uid)
    }
}

pub struct ExclusiveComponentRegistryAPI<'a> {
    pub(crate) manager: &'a mut ComponentRegistry,
}

impl<'a> ExclusiveComponentRegistryAPI<'a> {
    pub fn add_static<C: ComponentData>(&mut self, name: &str) -> Result<(), RegistryError> {
        self.manager.add_static::<C>(name)
    }

    pub fn find<H: ComponentHandle>(&self, component: UID) -> Option<H> {
        self.manager.find(component)
    }
}

pub struct ExclusiveRegistryAPI<'a> {
    pub systems: ExclusiveSystemRegistryAPI<'a>,
    pub components: ExclusiveComponentRegistryAPI<'a>,
}
