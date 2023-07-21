use self::{
    asset::ExclusiveAssetContext,
    event::EventContext,
    input::{ExclusiveInputContext, ParallelInputContext},
    registry::RegistryContext,
    renderer::{ExclusiveRendererContext, ParallelRendererContext},
    scene::{ExclusiveSceneContext, ParallelSceneContext},
    scheduler::SchedulerContext,
    signal::{ExclusiveSignalContext, ParallelSignalContext},
    time::TimeContext,
};

pub mod asset;
pub mod error;
pub mod event;
pub mod input;
pub mod registry;
pub mod renderer;
pub mod scene;
pub mod scheduler;
pub mod signal;
pub mod time;

pub struct ExclusiveContext<'a> {
    pub asset: ExclusiveAssetContext<'a>,
    pub event: EventContext<'a>,
    pub input: ExclusiveInputContext<'a>,
    pub signal: ExclusiveSignalContext<'a>,
    pub registry: RegistryContext<'a>,
    pub renderer: ExclusiveRendererContext<'a>,
    pub scene: ExclusiveSceneContext<'a>,
    pub scheduler: SchedulerContext<'a>,
    pub time: TimeContext,
}

impl<'a> ExclusiveContext<'a> {}

pub struct ParallelContext<'a> {
    pub asset: ExclusiveAssetContext<'a>,
    pub event: EventContext<'a>,
    pub input: ParallelInputContext<'a>,
    pub signal: ParallelSignalContext,
    pub registry: RegistryContext<'a>,
    pub renderer: ParallelRendererContext<'a>,
    pub scene: ParallelSceneContext<'a>,
    pub scheduler: SchedulerContext<'a>,
    pub time: TimeContext,
}

impl<'a> ParallelContext<'a> {}
