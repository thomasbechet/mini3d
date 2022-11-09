use anyhow::{Result, Context};
use slotmap::{new_key_type, SlotMap};

use crate::{backend::{renderer::RendererBackend, Backend}, asset::AssetManager, input::InputManager, event::AppEvents};

pub trait ProgramBuilder {
    type BuildData;
    fn build(id: ProgramId, data: Self::BuildData) -> Self;
}

pub trait Program {
    fn start(&mut self, ctx: &mut ProgramContext) -> Result<()>;
    fn update(&mut self, ctx: &mut ProgramContext) -> Result<()>;
    fn stop(&mut self, ctx: &mut ProgramContext) -> Result<()>;
}

new_key_type! { pub struct ProgramId; }

struct ProgramInstance {
    name: String,
    parent: ProgramId,
    program: Option<Box<dyn Program>>,
}

pub struct ProgramContext<'a> {
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub renderer: &'a mut dyn RendererBackend,
    pub events: &'a AppEvents,
    pub delta_time: f64,
}

impl<'a> ProgramContext<'a> {
    pub(crate) fn wrap(
        asset: &'a mut AssetManager,
        input: &'a mut InputManager,
        renderer: &'a mut dyn RendererBackend,
        events: &'a AppEvents,
        delta_time: f64,
    ) -> Self {
        Self {
            asset,
            input,
            renderer,
            events,
            delta_time,
        }
    }
}

#[derive(Default)]
pub(crate) struct ProgramManager {
    programs: SlotMap<ProgramId, ProgramInstance>,
    starting_programs: Vec<ProgramId>,
    stopping_programs: Vec<ProgramId>,
}

impl ProgramManager {

    pub(crate) fn run<P>(&mut self, name: &str, data: P::BuildData, parent: ProgramId) -> Result<ProgramId> 
        where P: Program + ProgramBuilder + 'static {
        // Prepare program instance
        let id = self.programs.insert(ProgramInstance {
            name: name.to_string(),
            parent,
            program: None,
        });
        // Build the program
        self.programs.get_mut(id).unwrap().program = Some(Box::new(P::build(id, data)));
        // Add program to starting programs
        self.starting_programs.push(id);
        // Return the id
        Ok(id)
    }

    pub(crate) fn update(
        &mut self, 
        asset: &mut AssetManager, 
        input: &mut InputManager, 
        backend: &mut Backend,
        events: &AppEvents,
        delta_time: f64
    ) -> Result<()> {
        
        // Create service wrapper
        let mut services = ProgramContext::wrap(asset, input, backend.renderer, events, delta_time);       

        // Start programs
        for id in self.starting_programs.drain(..) {
            let instance = self.programs.get_mut(id).unwrap();
            instance.program.as_mut().unwrap().start(&mut services)
                .context(format!("Failed to start program '{}'", instance.name))?;
        }

        // Update programs
        for (_, instance) in self.programs.iter_mut() {
            instance.program.as_mut().unwrap().update(&mut services)
                .context(format!("Failed to update program '{}'", instance.name))?;
        }

        // Stop and remove programs
        for id in self.starting_programs.drain(..) {
            let instance = self.programs.get_mut(id).unwrap();
            instance.program.as_mut().unwrap().stop(&mut services)
                .context(format!("Failed to stop program '{}'", instance.name))?;
            self.programs.remove(id);
        }

        Ok(())
    }
}