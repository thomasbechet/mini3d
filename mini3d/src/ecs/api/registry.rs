use crate::{
    registry::{
        component::{ComponentEntry, ComponentType},
        error::RegistryError,
    },
    utils::uid::ToUID,
};

use super::context::Context;

pub struct ResourceRegistry;

impl ResourceRegistry {
    pub fn add_static<C: Component>(
        ctx: &mut Context,
        name: &str,
    ) -> Result<StaticResourceType<D>, RegistryError> {
        ctx.registry.resource.add_static(name)
    }

    pub fn find<H: ResourceTypeTrait>(ctx: &Context, resource: impl ToUID) -> Option<H> {
        ctx.registry.resource.find(resource)
    }

    pub fn contains(ctx: &Context, resource: impl ToUID) -> bool {
        ctx.registry.resource.contains(resource)
    }
}

pub struct ComponentRegistry;

impl ComponentRegistry {
    pub fn add_static<C: Component>(
        ctx: &mut Context,
        name: &str,
        storage: ComponentStorage,
    ) -> Result<StaticComponentType<C>, RegistryError> {
        ctx.registry.component.add_static(name, storage)
    }

    pub fn add_dynamic(
        ctx: &mut Context,
        name: &str,
        storage: ComponentStorage,
    ) -> Result<ComponentType, RegistryError> {
        unimplemented!()
    }

    pub fn definition<'a, H: ComponentTypeTrait>(
        ctx: &'a Context,
        handle: H,
    ) -> Result<&'a ComponentEntry, RegistryError> {
        ctx.registry.component.definition(handle)
    }

    pub fn find<H: ComponentTypeTrait>(ctx: &Context, component: impl ToUID) -> Option<H> {
        ctx.registry.component.find(component)
    }

    pub fn contains(ctx: &Context, component: impl ToUID) -> bool {
        ctx.registry.component.contains(component)
    }
}

pub struct SystemRegistry;

impl SystemRegistry {
    pub fn add_static_exclusive<S: ExclusiveSystem>(
        ctx: &mut Context,
        name: &str,
        stage: &str,
        order: SystemOrder,
    ) -> Result<System, RegistryError> {
        ctx.registry
            .system
            .add_static_exclusive::<S>(name, stage, order)
    }

    pub fn add_static_parallel<S: ParallelSystem>(
        ctx: &mut Context,
        name: &str,
        stage: &str,
        order: SystemOrder,
    ) -> Result<System, RegistryError> {
        ctx.registry
            .system
            .add_static_parallel::<S>(name, stage, order)
    }

    pub fn find(ctx: &Context, system: impl ToUID) -> Option<System> {
        ctx.registry.system.find(system)
    }
}
