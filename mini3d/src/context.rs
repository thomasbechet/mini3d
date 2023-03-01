use self::{asset::AssetContext, renderer::RendererContext, input::InputContext, scheduler::SchedulerContext, procedure::ProcedureContext, world::WorldContext, registry::RegistryContext, event::EventContext};

pub mod asset;
pub mod event;
pub mod input;
pub mod procedure;
pub mod registry;
pub mod renderer;
pub mod scheduler;
pub mod time;
pub mod world;

pub struct SystemContext<'a> {
    pub asset: AssetContext<'a>,
    pub event: EventContext<'a>,
    pub input: InputContext<'a>,
    pub procedure: ProcedureContext<'a>,
    pub registry: RegistryContext<'a>,
    pub renderer: RendererContext<'a>,
    pub scheduler: SchedulerContext<'a>,
    pub time: time::TimeContext,
    pub world: WorldContext<'a>,
}