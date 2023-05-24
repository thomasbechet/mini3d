use crate::{context::SystemContext, script::ScriptManager, registry::{system::{SystemRegistry, SystemCode}, error::RegistryError}, uid::UID};

use super::system::SystemResult;

pub(crate) struct CompiledSystemPipeline {
    systems: Vec<SystemCode>,
}

impl CompiledSystemPipeline {

    pub(crate) fn build<'a>(registry: &SystemRegistry, systems: impl Iterator<Item = &'a UID>) -> Result<Self, RegistryError> {
        let mut codes = Vec::new();
        for uid in systems {
            let system = registry.get(uid).ok_or(RegistryError::SystemDefinitionNotFound { uid: *uid })?;
            codes.push(system.code);
        }
        Ok(Self { systems: codes })
    }

    pub(crate) fn run(&self, context: &mut SystemContext, _script: &ScriptManager) -> SystemResult {
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