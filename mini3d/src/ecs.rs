use crate::serialize::{Decoder, DecoderError, EncoderError, Serialize};
use core::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    asset::AssetManager,
    context::{
        asset::AssetContext, event::EventContext, input::InputContext, procedure::ProcedureContext,
        registry::RegistryContext, renderer::RendererContext, scheduler::SchedulerContext,
        time::TimeContext, world::WorldContext, SystemContext,
    },
    event::Events,
    input::InputManager,
    registry::{component::ComponentRegistry, RegistryManager},
    renderer::RendererManager,
    script::ScriptManager,
    serialize::Encoder,
    uid::UID,
};

use self::{
    error::ECSError, pipeline::CompiledSystemPipeline, procedure::Procedure, scheduler::Scheduler,
    world::World,
};

pub mod container;
pub mod entity;
pub mod error;
pub mod pipeline;
pub mod procedure;
pub mod query;
pub mod reference;
pub mod scheduler;
pub mod singleton;
pub mod sparse;
pub mod system;
pub mod view;
pub mod world;

pub(crate) struct ECSManager {
    scheduler: Scheduler,
    next_frame_system_invocations: Vec<UID>,
    next_frame_procedures: VecDeque<UID>,
    pub(crate) worlds: RefCell<HashMap<UID, RefCell<Box<World>>>>,
    pub(crate) active_world: UID,
}

impl Default for ECSManager {
    fn default() -> Self {
        Self {
            scheduler: Scheduler::default(),
            next_frame_system_invocations: Vec::new(),
            next_frame_procedures: VecDeque::new(),
            worlds: RefCell::new(HashMap::from([(
                Self::MAIN_WORLD.into(),
                RefCell::new(Box::new(World::new(Self::MAIN_WORLD))),
            )])),
            active_world: Self::MAIN_WORLD.into(),
        }
    }
}

pub(crate) struct ECSUpdateContext<'a> {
    pub(crate) registry: &'a RefCell<RegistryManager>,
    pub(crate) asset: &'a mut AssetManager,
    pub(crate) input: &'a mut InputManager,
    pub(crate) renderer: &'a mut RendererManager,
    pub(crate) events: &'a Events,
    pub(crate) delta_time: f64,
    pub(crate) time: f64,
    pub(crate) fixed_delta_time: f64,
}

#[allow(clippy::too_many_arguments)]
fn create_system_context<'b, 'a: 'b>(
    context: &'b mut ECSUpdateContext<'a>,
    active_procedure: UID,
    active_world: UID,
    scheduler: &'b mut Scheduler,
    frame_procedures: &'b mut VecDeque<UID>,
    next_frame_procedures: &'b mut VecDeque<UID>,
    worlds: &'b mut HashMap<UID, RefCell<Box<World>>>,
    change_world: &'b mut Option<UID>,
    removed_worlds: &'b mut HashSet<UID>,
) -> SystemContext<'b> {
    SystemContext {
        asset: AssetContext {
            registry: context.registry,
            manager: context.asset,
        },
        event: EventContext {
            events: context.events,
        },
        input: InputContext {
            manager: context.input,
        },
        procedure: ProcedureContext {
            active_procedure,
            frame_procedures,
            next_frame_procedures,
        },
        registry: RegistryContext {
            manager: context.registry,
        },
        renderer: RendererContext {
            manager: context.renderer,
        },
        scheduler: SchedulerContext { scheduler },
        time: TimeContext {
            delta: if active_procedure == Procedure::FIXED_UPDATE.into() {
                context.fixed_delta_time
            } else {
                context.delta_time
            },
            global: context.time,
        },
        world: WorldContext {
            registry: context.registry,
            worlds,
            active_world,
            change_world,
            removed_worlds,
        },
    }
}

impl ECSManager {
    const MAIN_WORLD: &'static str = "main";

    pub(crate) fn save_state(&self, encoder: &mut impl Encoder) -> Result<(), EncoderError> {
        // Scheduler
        self.scheduler.serialize(encoder)?;
        // Next frame system invocations
        self.next_frame_system_invocations.serialize(encoder)?;
        // Next frame procedures
        self.next_frame_procedures.serialize(encoder)?;
        // Worlds
        encoder.write_u32(self.worlds.borrow().len() as u32)?;
        for world in self.worlds.borrow().values() {
            world.borrow().serialize(encoder)?;
        }
        // Active world
        self.active_world.serialize(encoder)?;
        Ok(())
    }

    pub(crate) fn load_state(
        &mut self,
        registry: &ComponentRegistry,
        decoder: &mut impl Decoder,
    ) -> Result<(), DecoderError> {
        // Scheduler
        self.scheduler = Scheduler::deserialize(decoder, &Default::default())?;
        // Next frame system invocations
        self.next_frame_system_invocations = Vec::<UID>::deserialize(decoder, &Default::default())?;
        // Next frame procedures
        self.next_frame_procedures = VecDeque::<UID>::deserialize(decoder, &Default::default())?;
        // Worlds
        let worlds_count = decoder.read_u32()?;
        for _ in 0..worlds_count {
            let world = World::deserialize(registry, decoder)?;
            self.worlds
                .borrow_mut()
                .insert(UID::new(&world.name), RefCell::new(Box::new(world)));
        }
        // Active world
        self.active_world = UID::deserialize(decoder, &Default::default())?;
        Ok(())
    }

    pub(crate) fn invoke(&mut self, system: UID) {
        self.next_frame_system_invocations.push(system)
    }

    pub(crate) fn update(
        &mut self,
        mut context: ECSUpdateContext,
        scripts: &mut ScriptManager,
        fixed_update_count: u32,
    ) -> Result<(), ECSError> {
        // Prepare frame
        let mut change_world: Option<UID> = None;
        let mut removed_worlds: HashSet<UID> = Default::default();
        let mut worlds = self.worlds.borrow_mut();

        // Collect procedures
        let mut frame_procedures = self
            .next_frame_procedures
            .drain(..)
            .collect::<VecDeque<_>>();
        for _ in 0..fixed_update_count {
            frame_procedures.push_back(Procedure::FIXED_UPDATE.into());
        }
        frame_procedures.push_back(Procedure::UPDATE.into());

        // Invoke frame systems
        if !self.next_frame_system_invocations.is_empty() {
            // Build context
            let pipeline = CompiledSystemPipeline::build(
                &context.registry.borrow_mut().systems,
                self.next_frame_system_invocations.iter(),
            )
            .map_err(|_| ECSError::RegistryError)?;
            pipeline
                .run(
                    &mut create_system_context(
                        &mut context,
                        UID::null(),
                        self.active_world,
                        &mut self.scheduler,
                        &mut frame_procedures,
                        &mut self.next_frame_procedures,
                        &mut worlds,
                        &mut change_world,
                        &mut removed_worlds,
                    ),
                    scripts,
                )
                .map_err(|_| ECSError::SystemError)?;
            self.next_frame_system_invocations.clear();
        }

        // Run procedures
        // TODO: protect against infinite loop
        while let Some(procedure) = frame_procedures.pop_front() {
            // Build pipeline
            if let Some(pipeline) = self
                .scheduler
                .build_pipeline(procedure, context.registry)
                .map_err(|_| ECSError::RegistryError)?
            {
                // Run pipeline
                pipeline
                    .run(
                        &mut create_system_context(
                            &mut context,
                            procedure,
                            self.active_world,
                            &mut self.scheduler,
                            &mut frame_procedures,
                            &mut self.next_frame_procedures,
                            &mut worlds,
                            &mut change_world,
                            &mut removed_worlds,
                        ),
                        scripts,
                    )
                    .map_err(|_| ECSError::RegistryError)?;
            }

            // Remove worlds
            for uid in removed_worlds.drain() {
                self.worlds.borrow_mut().remove(&uid);
            }

            // Change world
            if let Some(world) = change_world {
                self.active_world = world;
                self.next_frame_procedures
                    .push_front(Procedure::WORLD_CHANGED.into());
                change_world = None;
            }
        }

        Ok(())
    }
}
