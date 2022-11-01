use rhai::exported_module;
use anyhow::{Result, anyhow};
use slotmap::SecondaryMap;

use crate::asset::{rhai_script::{RhaiScriptId, RhaiScript}, AssetEntry};

use self::{script_storage::rhai_script_storage_api, input::rhai_input_api, world::rhai_world_api};

pub mod input;
pub mod script_storage;
pub mod world;

pub struct RhaiContext {
    pub engine: rhai::Engine,
    scripts: SecondaryMap<RhaiScriptId, rhai::AST>,
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

    fn check_compiled(&mut self, entry: &AssetEntry<RhaiScript>) -> Result<()> {
        if !self.scripts.contains_key(entry.id) {
            let ast = self.engine.compile(entry.asset.source.clone())?;
            self.scripts.insert(entry.id, ast);
        }
        Ok(())
    }

    pub fn call(&mut self, entry: &AssetEntry<RhaiScript>, scope: &mut rhai::Scope, function: &str) -> Result<()> {
        // Check compilation
        self.check_compiled(entry)?;
        // Call script
        if let Some(ast) = self.scripts.get(entry.id) {
            self.engine.call_fn::<()>(scope, ast, function, ()).map_err(|err| {
                anyhow!(err.to_string())
            })?;
        }
        Ok(())
    }
}