use self::{asset::AssetContext, renderer::RendererContext, input::InputContext, scheduler::SchedulerContext, procedure::ProcedureContext, world::WorldContext, registry::RegistryContext};

pub mod asset;
pub mod input;
pub mod procedure;
pub mod registry;
pub mod renderer;
pub mod scheduler;
pub mod world;

pub struct SystemContext<'a> {

    // Context
    pub asset: AssetContext<'a>,
    pub input: InputContext<'a>,
    pub procedure: ProcedureContext<'a>,
    pub registry: RegistryContext<'a>,
    pub renderer: RendererContext<'a>,
    pub scheduler: SchedulerContext<'a>,
    pub world: WorldContext<'a>,

    // Time
    pub delta_time: f64,
    pub time: f64,
}