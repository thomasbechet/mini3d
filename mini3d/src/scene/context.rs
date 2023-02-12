use std::collections::{HashMap, VecDeque};

use anyhow::Result;

use crate::{asset::AssetManager, input::InputManager, script::ScriptManager, renderer::RendererManager, uid::UID, feature::asset::{world_template::WorldTemplate, schedule::Schedule}, registry::RegistryManager};

use super::{SceneInfo, Scene, world::World, entity::Entity, view::{ComponentView, ComponentViewMut}, query::Query};

pub(crate) enum SceneCommand {
    Load(UID, Box<Scene>),
    Unload(UID),
    Change(UID),
}

pub struct SystemContext<'a> {

    registry: &'a mut RegistryManager,
    asset: &'a mut AssetManager,
    input: &'a mut InputManager,
    renderer: &'a mut RendererManager,

    delta_time: f64,
    time: f64,
    scene: UID,

    scene_info: &'a mut HashMap<UID, SceneInfo>,
    scene_commands: &'a mut Vec<SceneCommand>,

    signal_queue: &'a mut VecDeque<UID>,

    pub(crate) script: &'a mut ScriptManager,
    world: &'a mut World,
}

impl<'a> SystemContext<'a> {
    
}