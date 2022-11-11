use std::collections::HashMap;

use anyhow::{anyhow, Result};

use crate::{uid::UID, ecs::system::{System, despawn::DespawnEntitiesSystem, free_fly::FreeFlySystem, renderer::{RendererCheckLifecycleSystem, RendererTransferTransformsSystem, RendererUpdateCameraSystem}, rhai::RhaiUpdateScriptsSystem, rotator::RotatorSystem}};

pub(crate) struct SystemEntry {
    pub(crate) name: String,
    pub(crate) system: Box<dyn System>,
}

pub struct SystemManager {
    pub(crate) systems: HashMap<UID, SystemEntry>, 
}

impl Default for SystemManager {
    fn default() -> Self {
        let mut manager = Self { systems: HashMap::default() };
        manager.register("despawn_entities", DespawnEntitiesSystem {}).unwrap();
        manager.register("free_fly", FreeFlySystem {}).unwrap();
        manager.register("renderer_check_lifecycle", RendererCheckLifecycleSystem {}).unwrap();
        manager.register("renderer_transfer_transforms", RendererTransferTransformsSystem {}).unwrap();
        manager.register("renderer_update_camera", RendererUpdateCameraSystem {}).unwrap();
        manager.register("rhai_update_scripts", RhaiUpdateScriptsSystem {}).unwrap();
        manager.register("rotator", RotatorSystem {}).unwrap();
        manager
    }
}

impl SystemManager {
    pub fn register<S: System + 'static>(&mut self, name: &str, system: S) -> Result<()> {
        let uid: UID = name.into();
        if self.systems.contains_key(&uid) {
            return Err(anyhow!("System '{}' already exists", name));
        }
        self.systems.insert(uid, SystemEntry { 
            name: name.to_string(),
            system: Box::new(system) 
        });
        Ok(())
    }
}