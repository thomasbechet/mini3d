use crate::uid::UID;

pub struct SceneContext<'a> {
    
}

impl<'a> SceneContext<'a> {

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