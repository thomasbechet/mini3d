use std::collections::{HashMap, hash_map};

use rhai::exported_module;
use anyhow::{Result, anyhow, Context};

use crate::{asset::{rhai_script::RhaiScript, AssetManager}, uid::UID};

use self::{script_storage::rhai_script_storage_api, input::rhai_input_api, world::rhai_world_api};

pub mod input;
pub mod script_storage;
pub mod world;

pub struct RhaiScriptCache {
    pub engine: rhai::Engine,
    scripts: HashMap<UID, rhai::AST>,
}

impl Default for RhaiScriptCache {
    fn default() -> Self {
        let mut cache = Self {
            engine: rhai::Engine::new(),
            scripts: Default::default(),
        };
        cache.engine.register_global_module(exported_module!(rhai_script_storage_api).into());
        cache.engine.register_global_module(exported_module!(rhai_input_api).into());
        cache.engine.register_global_module(exported_module!(rhai_world_api).into());
        cache
    }
}

impl RhaiScriptCache {
    pub fn call(&mut self, uid: UID, asset: &AssetManager, scope: &mut rhai::Scope, function: &str) -> Result<()> {
        // Lazy script compilation
        if let hash_map::Entry::Vacant(e) = self.scripts.entry(uid) {
            let script = asset.get::<RhaiScript>(uid)
                .context("Rhai script not found")?;
            let ast = self.engine.compile(script.source.clone())?;
            e.insert(ast);
        }
        // Call script
        if let Some(ast) = self.scripts.get(&uid) {
            self.engine.call_fn::<()>(scope, ast, function, ()).map_err(|err| {
                anyhow!(err.to_string())
            })?;
        }
        Ok(())
    }
}