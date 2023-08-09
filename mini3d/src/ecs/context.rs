use self::{
    asset::{ExclusiveAssetContext, ParallelAssetContext},
    event::EventContext,
    input::{ExclusiveInputContext, ParallelInputContext},
    registry::RegistryContext,
    renderer::{ExclusiveRendererContext, ParallelRendererContext},
    scene::{ExclusiveSceneContext, ParallelSceneContext},
    time::TimeContext,
};

pub mod asset;
pub mod error;
pub mod event;
pub mod input;
pub mod registry;
pub mod renderer;
pub mod scene;
pub mod time;

pub struct ExclusiveContext<'a> {
    pub asset: ExclusiveAssetContext<'a>,
    pub event: EventContext<'a>,
    pub input: ExclusiveInputContext<'a>,
    pub registry: RegistryContext<'a>,
    pub renderer: ExclusiveRendererContext<'a>,
    pub scene: ExclusiveSceneContext<'a>,
    pub time: TimeContext,
}

pub struct ParallelContext<'a> {
    pub asset: ParallelAssetContext<'a>,
    pub event: EventContext<'a>,
    pub input: ParallelInputContext<'a>,
    pub registry: RegistryContext<'a>,
    pub renderer: ParallelRendererContext<'a>,
    pub scene: ParallelSceneContext<'a>,
    pub time: TimeContext,
}
