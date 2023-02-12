use std::collections::HashMap;

use anyhow::Result;

use crate::{uid::UID, feature::asset::{world_template::WorldTemplate, schedule::Schedule}, scene::{SceneInfo, Scene, world::World}};

pub(crate) enum SceneCommand {
    Load(UID, Box<Scene>),
    Unload(UID),
    Change(UID),
}

pub struct SceneContext<'a> {
    scene_info: &'a HashMap<UID, SceneInfo>,
    scene_commands: &'a mut Vec<SceneCommand>,
}

impl<'a> SceneContext<'a> {

    pub(crate) fn new(scene_info: &'a HashMap<UID, SceneInfo>, scene_commands: &'a mut Vec<SceneCommand>) -> Self {
        Self { scene_info, scene_commands }
    }

    pub fn load(&mut self, name: &str, world: WorldTemplate, schedule: Schedule) -> Result<()> {
        let uid: UID = name.into();
        if self.scene_info.contains_key(&uid) {
            return Err(anyhow::anyhow!("Scene with name {} already exists", name));
        }
        self.scene_info.insert(uid, SceneInfo { name: name.to_string(), index: 0 });
        self.scene_commands.push(SceneCommand::Load(uid, Box::new(Scene::new(World::default(), schedule))));
        Ok(())
    }
    
    pub fn unload(&mut self, uid: UID) -> Result<()> {
        if !self.scene_info.contains_key(&uid) {
            return Err(anyhow::anyhow!("Scene with uid {} does not exist", uid));
        }
        self.scene_commands.push(SceneCommand::Unload(uid));
        Ok(())
    }

    pub fn change(&mut self, uid: UID) -> Result<()> {
        if !self.scene_info.contains_key(&uid) {
            return Err(anyhow::anyhow!("Scene with uid {} does not exist", uid));
        }
        self.scene_commands.push(SceneCommand::Change(uid));
        Ok(())
    }
}