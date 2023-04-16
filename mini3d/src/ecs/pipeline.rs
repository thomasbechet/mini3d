use anyhow::{Result, Context};

use crate::{context::SystemContext, script::ScriptManager, registry::system::{SystemRegistry, SystemCode}, uid::UID};

pub(crate) struct CompiledSystemPipeline {
    systems: Vec<SystemCode>,
}

impl CompiledSystemPipeline {

    pub(crate) fn build<'a>(registry: &SystemRegistry, systems: impl Iterator<Item = &'a UID>) -> Result<Self> {
        let mut codes = Vec::new();
        for uid in systems {
            let system = registry.get(uid).with_context(|| "System not found in registry")?;
            codes.push(system.code);
        }
        Ok(Self { systems: codes })
    }

    pub(crate) fn run(&self, context: &mut SystemContext, _script: &ScriptManager) -> Result<()> {
        for system in &self.systems {
            match system {
                SystemCode::Static(callback) => callback(context)?,
                SystemCode::Script(_uid) => {
                    todo!()
                },
            }
        }
        Ok(())
    }
}