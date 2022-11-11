use anyhow::Result;
use hecs::World;

use crate::{asset::AssetManager, input::InputManager, script::ScriptManager, backend::renderer::RendererBackend};

pub mod despawn;
pub mod free_fly;
pub mod renderer;
pub mod rhai;
pub mod rotator;

pub struct SystemContext<'a> {
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub script: &'a mut ScriptManager,
    pub renderer: &'a mut dyn RendererBackend,
    pub delta_time: f64,
}

pub trait System {
    fn run(&self, ctx: &mut SystemContext, world: &mut World) -> Result<()>;
}