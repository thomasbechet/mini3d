use self::{
    asset::{ExclusiveAssetAPI, ParallelAssetAPI},
    input::{ExclusiveInputAPI, ParallelInputAPI},
    registry::{ExclusiveRegistryAPI, ParallelRegistryAPI},
    renderer::{ExclusiveRendererAPI, ParallelRendererAPI},
    system::{ExclusiveSystemAPI, ParallelSystemAPI},
    time::TimeAPI,
};

pub mod asset;
pub mod ecs;
pub mod error;
pub mod input;
pub mod registry;
pub mod renderer;
pub mod system;
pub mod time;

pub struct ExclusiveAPI<'a> {
    pub asset: ExclusiveAssetAPI<'a>,
    pub input: ExclusiveInputAPI<'a>,
    pub registry: ExclusiveRegistryAPI<'a>,
    pub renderer: ExclusiveRendererAPI<'a>,
    pub system: ExclusiveSystemAPI<'a>,
    pub time: TimeAPI,
}

pub struct ParallelAPI<'a> {
    pub asset: ParallelAssetAPI<'a>,
    pub input: ParallelInputAPI<'a>,
    pub registry: ParallelRegistryAPI<'a>,
    pub renderer: ParallelRendererAPI<'a>,
    pub system: ParallelSystemAPI<'a>,
    pub time: TimeAPI,
}
