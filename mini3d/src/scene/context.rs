use std::collections::{HashMap, VecDeque};

use anyhow::Result;

use crate::{asset::AssetManager, input::InputManager, script::ScriptManager, renderer::RendererManager, uid::UID, feature::asset::{world_template::WorldTemplate, schedule::Schedule}};

use super::{SceneInfo, Scene, world::World};

pub(crate) enum SceneProxyCommand {
    Load(UID, Box<Scene>),
    Unload(UID),
    Change(UID),
}

pub struct SceneProxy<'a> {
    pub current: UID,
    scene_info: &'a mut HashMap<UID, SceneInfo>,
    commands: &'a mut Vec<SceneProxyCommand>,
}

impl<'a> SceneProxy<'a> {

    pub fn load(&mut self, name: &str, world: WorldTemplate, schedule: Schedule) -> Result<()> {
        let uid: UID = name.into();
        if self.scene_info.contains_key(&uid) {
            return Err(anyhow::anyhow!("Scene with name {} already exists", name));
        }
        self.scene_info.insert(uid, SceneInfo { name: name.to_string(), index: 0 });
        self.commands.push(SceneProxyCommand::Load(uid, Box::new(Scene::new(World::default(), schedule))));
        Ok(())
    }
    pub fn unload(&mut self, uid: UID) -> Result<()> {
        if !self.scene_info.contains_key(&uid) {
            return Err(anyhow::anyhow!("Scene with uid {} does not exist", uid));
        }
        self.commands.push(SceneProxyCommand::Unload(uid));
        Ok(())
    }
    pub fn change(&mut self, uid: UID) -> Result<()> {
        if !self.scene_info.contains_key(&uid) {
            return Err(anyhow::anyhow!("Scene with uid {} does not exist", uid));
        }
        self.commands.push(SceneProxyCommand::Change(uid));
        Ok(())
    }
}

pub struct SignalProxy<'a> {
    pub(crate) signal_queue: &'a mut VecDeque<UID>,
}

impl<'a> SignalProxy<'a> {
    pub fn emit(&mut self, uid: UID) {
        self.signal_queue.push_back(uid);
    }
}

pub struct SystemContext<'a> {
    pub asset: &'a mut AssetManager,
    pub input: &'a mut InputManager,
    pub script: &'a mut ScriptManager,
    pub renderer: &'a mut RendererManager,
    pub scene: &'a mut SceneProxy<'a>,
    pub signal: &'a mut SignalProxy<'a>,
    pub delta_time: f64,
    pub time: f64,
}