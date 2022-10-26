use std::collections::HashMap;

use rhai::exported_module;
use anyhow::Result;

use crate::asset::AssetUID;

use self::{script_storage::rhai_script_storage_api, input::rhai_input_api, world::rhai_world_api};

pub mod input;
pub mod script_storage;
pub mod world;

pub struct RhaiContext {
    pub engine: rhai::Engine,
    pub scripts: HashMap<AssetUID, rhai::AST>,
}

impl RhaiContext {

    pub fn new() -> Self {
        let mut context = Self {
            engine: rhai::Engine::new(),
            scripts: Default::default(),
        };
        context.engine.register_global_module(exported_module!(rhai_script_storage_api).into());
        context.engine.register_global_module(exported_module!(rhai_input_api).into());
        context.engine.register_global_module(exported_module!(rhai_world_api).into());
        context
    }

    pub fn compile(&mut self, uid: AssetUID, script: &str) -> Result<()> {
        self.scripts.insert(uid, self.engine.compile(script)?);
        Ok(())
    }
}