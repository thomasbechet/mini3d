use crate::{
    serialize::{Decoder, DecoderError, EncoderError, Serialize},
    utils::uid::UID,
};
use core::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    asset::AssetManager,
    context::{
        asset::AssetContext, event::EventContext, input::InputContext, procedure::ProcedureContext,
        registry::RegistryContext, renderer::RendererContext, scene::SceneContext,
        scheduler::SchedulerContext, time::TimeContext, ExclusiveSystemContext,
    },
    event::Events,
    input::InputManager,
    registry::{component::ComponentRegistry, RegistryManager},
    renderer::RendererManager,
    serialize::Encoder,
};

use self::{
    error::ECSError, pipeline::SystemPipeline, procedure::Procedure, scene::Scene,
    scheduler::Scheduler,
};

pub mod container;
pub mod entity;
pub mod error;
pub mod pipeline;
pub mod procedure;
pub mod query;
pub mod reference;
pub mod scene;
pub mod scheduler;
pub mod singleton;
pub mod sparse;
pub mod system;
pub mod view;

pub(crate) struct ECSManager {
    scheduler: Scheduler,
    next_frame_system_invocations: Vec<UID>,
    next_frame_procedures: VecDeque<UID>,
    pub(crate) scenes: RefCell<HashMap<UID, RefCell<Box<Scene>>>>,
    pub(crate) active_scene: UID,
}

impl Default for ECSManager {
    fn default() -> Self {
        Self {
            scheduler: Scheduler::default(),
            next_frame_system_invocations: Vec::new(),
            next_frame_procedures: VecDeque::new(),
            scenes: RefCell::new(HashMap::from([(
                Self::MAIN_SCENE.into(),
                RefCell::new(Box::new(Scene::new(Self::MAIN_SCENE))),
            )])),
            active_scene: Self::MAIN_SCENE.into(),
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
    active_scene: UID,
    scheduler: &'b mut Scheduler,
    frame_procedures: &'b mut VecDeque<UID>,
    next_frame_procedures: &'b mut VecDeque<UID>,
    scenes: &'b mut HashMap<UID, RefCell<Box<Scene>>>,
    change_scene: &'b mut Option<UID>,
    removed_scenes: &'b mut HashSet<UID>,
) -> ExclusiveSystemContext<'b> {
    ExclusiveSystemContext {
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
        scene: SceneContext {
            registry: context.registry,
            scenes,
            active_scene,
            change_scene,
            removed_scenes,
        },
    }
}

impl ECSManager {
    const MAIN_SCENE: &'static str = "main";

    pub(crate) fn save_state(
        &self,
        registry: &ComponentRegistry,
        encoder: &mut impl Encoder,
    ) -> Result<(), EncoderError> {
        // Scheduler
        self.scheduler.serialize(encoder)?;
        // Next frame system invocations
        self.next_frame_system_invocations.serialize(encoder)?;
        // Next frame procedures
        self.next_frame_procedures.serialize(encoder)?;
        // Scenes
        encoder.write_u32(self.scenes.borrow().len() as u32)?;
        for scene in self.scenes.borrow().values() {
            scene.borrow().serialize(registry, encoder)?;
        }
        // Active scene
        self.active_scene.serialize(encoder)?;
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
        // Scenes
        let scenes_count = decoder.read_u32()?;
        for _ in 0..scenes_count {
            let scene = Scene::deserialize(registry, decoder)?;
            self.scenes
                .borrow_mut()
                .insert(UID::new(&scene.name), RefCell::new(Box::new(scene)));
        }
        // Active scene
        self.active_scene = UID::deserialize(decoder, &Default::default())?;
        Ok(())
    }

    pub(crate) fn invoke(&mut self, system: UID) {
        self.next_frame_system_invocations.push(system)
    }

    pub(crate) fn update(
        &mut self,
        mut context: ECSUpdateContext,
        fixed_update_count: u32,
    ) -> Result<(), ECSError> {
        // Prepare frame
        let mut change_scene: Option<UID> = None;
        let mut removed_scenes: HashSet<UID> = Default::default();
        let mut scenes = self.scenes.borrow_mut();

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
            let pipeline = SystemPipeline::build(
                &context.registry.borrow_mut().systems,
                self.next_frame_system_invocations.iter(),
            )
            .map_err(|_| ECSError::RegistryError)?;
            pipeline
                .run(&mut create_system_context(
                    &mut context,
                    UID::null(),
                    self.active_scene,
                    &mut self.scheduler,
                    &mut frame_procedures,
                    &mut self.next_frame_procedures,
                    &mut scenes,
                    &mut change_scene,
                    &mut removed_scenes,
                ))
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
                    .run(&mut create_system_context(
                        &mut context,
                        procedure,
                        self.active_scene,
                        &mut self.scheduler,
                        &mut frame_procedures,
                        &mut self.next_frame_procedures,
                        &mut scenes,
                        &mut change_scene,
                        &mut removed_scenes,
                    ))
                    .map_err(|_| ECSError::RegistryError)?;
            }

            // Remove scenes
            for uid in removed_scenes.drain() {
                self.scenes.borrow_mut().remove(&uid);
            }

            // Change scene
            if let Some(scene) = change_scene {
                self.active_scene = scene;
                self.next_frame_procedures
                    .push_front(Procedure::SCENE_CHANGED.into());
                change_scene = None;
            }
        }

        Ok(())
    }
}
