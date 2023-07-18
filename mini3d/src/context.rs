use self::{
    asset::AssetContext,
    event::EventContext,
    input::InputContext,
    procedure::ProcedureContext,
    registry::RegistryContext,
    renderer::RendererContext,
    scene::{ExclusiveSceneContext, ParallelSceneContext, SceneContext},
    scheduler::SchedulerContext,
    time::TimeContext,
};

pub mod asset;
pub mod error;
pub mod event;
pub mod input;
pub mod procedure;
pub mod registry;
pub mod renderer;
pub mod scene;
pub mod scheduler;
pub mod time;

// pub struct ExclusiveSystemContext<'a> {
//     pub asset: AssetContext<'a>,
//     pub event: EventContext<'a>,
//     pub input: InputContext<'a>,
//     pub procedure: ProcedureContext<'a>,
//     pub registry: RegistryContext<'a>,
//     pub renderer: RendererContext<'a>,
//     pub scheduler: SchedulerContext<'a>,
//     pub time: TimeContext,
//     pub scene: ExclusiveSceneContext<'a>,
// }

pub struct ExclusiveContext<'a> {
    pub scene: ExclusiveSceneContext<'a>,
    pub input: InputContext<'a>,
    pub event: EventContext<'a>,
    pub time: TimeContext,
}

pub struct ParallelContext<'a> {
    pub scene: ParallelSceneContext<'a>,
    pub input: InputContext<'a>,
    pub event: EventContext<'a>,
    pub time: TimeContext,
}
