use anyhow::{Result, Context};
use slotmap::{new_key_type, SlotMap};

use crate::{backend::{renderer::RendererBackend, Backend}, asset::AssetManager, input::InputManager};

pub trait ProgramBuilder {
    type BuildData;
    fn build(id: ProgramId, data: Self::BuildData) -> Self;
}

pub trait Program {
    fn start(&mut self, ctx: &mut ProgramContext) -> Result<()>;
    fn update(&mut self, ctx: &mut ProgramContext) -> Result<()>;
    fn stop(&mut self, ctx: &mut ProgramContext) -> Result<()>;
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ProgramStatus {
    STARTING,
    RUNNING,
    STOPPING,
}

new_key_type! { pub struct ProgramId; }

struct ProgramInstance {
    name: String,
    parent: Option<ProgramId>,
    program: Option<Box<dyn Program>>,
    status: ProgramStatus,
}

pub struct ProgramContext<'a> {
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub renderer: &'a mut dyn RendererBackend,
}

impl<'a> ProgramContext<'a> {
    pub(crate) fn wrap(
        asset: &'a mut AssetManager,
        input: &'a mut InputManager,
        renderer: &'a mut dyn RendererBackend,
    ) -> Self {
        Self {
            asset,
            input,
            renderer,
        }
    }
}

#[derive(Default)]
pub(crate) struct ProgramManager {
    programs: SlotMap<ProgramId, ProgramInstance>,
    active_program: ProgramId,
}

impl ProgramManager {

    pub(crate) fn run<P>(&mut self, name: &str, data: P::BuildData, parent: Option<ProgramId>) -> Result<ProgramId> 
        where P: Program + ProgramBuilder + 'static {
        // Prepare program instance
        let id = self.programs.insert(ProgramInstance {
            name: name.to_string(),
            parent,
            program: None,
            status: ProgramStatus::STARTING,
        });
        // Build the program
        self.programs.get_mut(id).unwrap().program = Some(Box::new(P::build(id, data)));
        // Set program as active
        self.active_program = id;
        // Return the id
        Ok(id)
    }

    pub(crate) fn update(&mut self, asset: &mut AssetManager, input: &mut InputManager, backend: &mut Backend) -> Result<()> {
        
        // Create service wrapper
        let mut services = ProgramContext::wrap(asset, input, backend.renderer);

        // Collect program status before progress to resolve ABA problem
        let programs = self.programs.iter()
            .map(|(id, program)| (id, program.status))
            .collect::<Vec<_>>();        

        // Start, stop or update programs
        for (id, status) in &programs {
            let instance = self.programs.get_mut(*id).unwrap();
            match status {
                ProgramStatus::STARTING => {
                    instance.program.as_mut().unwrap().start(&mut services)
                        .context(format!("Failed to start program '{}'", instance.name))?;
                    instance.status = ProgramStatus::RUNNING;
                }
                ProgramStatus::STOPPING => {
                    instance.program.as_mut().unwrap().stop(&mut services)
                        .context(format!("Failed to stop program '{}'", instance.name))?;
                }
                ProgramStatus::RUNNING => {
                    instance.program.as_mut().unwrap().update(&mut services)
                       .context(format!("Failed to update program '{}'", instance.name))?;
                }
            }
        }

        // Remove stopped programs
        for (id, _) in programs.iter().filter(|(_, status)| *status == ProgramStatus::STOPPING) {
            self.programs.remove(*id);
        }

        Ok(())
    }
}