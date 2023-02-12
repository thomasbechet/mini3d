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

    pub fn input(&self) -> &InputManager {
        self.input
    }
    pub fn input_mut(&mut self) -> &mut InputManager {
        self.input
    }
    pub fn asset(&self) -> &AssetManager {
        self.asset
    }
    pub fn asset_mut(&mut self) -> &mut AssetManager {
        self.asset
    }
    pub fn renderer(&self) -> &RendererManager {
        self.renderer
    }
    pub fn renderer_mut(&mut self) -> &mut RendererManager {
        self.renderer
    }

    pub fn delta_time(&self) -> f64 {
        self.delta_time
    }
    pub fn time(&self) -> f64 {
        self.time
    }
    pub fn active_scene(&self) -> UID {
        self.scene
    }
    
    pub fn load_scene(&mut self, name: &str, world: WorldTemplate, schedule: Schedule) -> Result<()> {
        let uid: UID = name.into();
        if self.scene_info.contains_key(&uid) {
            return Err(anyhow::anyhow!("Scene with name {} already exists", name));
        }
        self.scene_info.insert(uid, SceneInfo { name: name.to_string(), index: 0 });
        self.scene_commands.push(SceneCommand::Load(uid, Box::new(Scene::new(World::default(), schedule))));
        Ok(())
    }
    pub fn unload_scene(&mut self, uid: UID) -> Result<()> {
        if !self.scene_info.contains_key(&uid) {
            return Err(anyhow::anyhow!("Scene with uid {} does not exist", uid));
        }
        self.scene_commands.push(SceneCommand::Unload(uid));
        Ok(())
    }
    pub fn change_scene(&mut self, uid: UID) -> Result<()> {
        if !self.scene_info.contains_key(&uid) {
            return Err(anyhow::anyhow!("Scene with uid {} does not exist", uid));
        }
        self.scene_commands.push(SceneCommand::Change(uid));
        Ok(())
    }

    pub fn emit_signal(&mut self, uid: UID) {
        self.signal_queue.push_back(uid);
    }

    pub fn add_entity(&mut self) -> Entity {

    }
    pub fn destroy_entity(&mut self, entity: Entity) -> Result<()> {

    }

    pub fn add_component<C: Component>(&mut self, entity: Entity, component: UID, data: C) -> Result<()> {

    }
    pub fn remove_component(&mut self, entity: Entity, component: UID) -> Result<()> {

    }

    pub fn component_view<'a, C: Component>(&'a self, component: UID) -> Result<ComponentView<'a, C>> {

    }
    pub fn component_view_mut<'a, C: Component>(&'a self, component: UID) -> Result<ComponentViewMut<'a, C>> {

    }

    pub fn query<'a>(&'a self, components: &[UID]) -> Query<'a> {

    }
}