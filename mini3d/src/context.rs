use self::{
    asset::AssetContext, event::EventContext, input::InputContext, procedure::ProcedureContext,
    registry::RegistryContext, renderer::RendererContext, scene::SceneContext,
    scheduler::SchedulerContext, time::TimeContext,
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

pub struct ExclusiveSystemContext<'a> {
    pub asset: AssetContext<'a>,
    pub event: EventContext<'a>,
    pub input: InputContext<'a>,
    pub procedure: ProcedureContext<'a>,
    pub registry: RegistryContext<'a>,
    pub renderer: RendererContext<'a>,
    pub scheduler: SchedulerContext<'a>,
    pub time: TimeContext,
    pub scene: SceneContext<'a>,
}
