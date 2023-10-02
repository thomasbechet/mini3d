use crate::{
    registry::{
        asset::{AssetTypeTrait, StaticAssetType},
        component::{
            ComponentEntry, ComponentStorage, ComponentType, ComponentTypeTrait,
            StaticComponentType,
        },
        datatype::StaticDataType,
        error::RegistryError,
        system::{ExclusiveSystem, ParallelSystem, System, SystemOrder},
    },
    utils::uid::ToUID,
};

use super::context::Context;

pub struct AssetRegistry;

impl AssetRegistry {
    pub fn add_static<D: StaticDataType>(
        ctx: &mut Context,
        name: &str,
    ) -> Result<StaticAssetType<D>, RegistryError> {
        ctx.registry.asset.add_static(name)
    }

    pub fn find<H: AssetTypeTrait>(ctx: &Context, asset: impl ToUID) -> Option<H> {
        ctx.registry.asset.find(asset)
    }

    pub fn contains(ctx: &Context, asset: impl ToUID) -> bool {
        ctx.registry.asset.contains(asset)
    }
}

pub struct ComponentRegistry;

impl ComponentRegistry {
    pub fn add_static<D: StaticDataType>(
        ctx: &mut Context,
        name: &str,
        storage: ComponentStorage,
    ) -> Result<StaticComponentType<D>, RegistryError> {
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