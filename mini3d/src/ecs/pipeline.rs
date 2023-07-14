use crate::{
    context::SystemContext,
    registry::{
        error::RegistryError,
        system::{SystemCode, SystemRegistry},
    },
    script::ScriptManager,
    uid::UID,
};

use super::system::SystemResult;

pub(crate) struct SystemPipeline {
    systems: Vec<SystemCode>,
}

impl SystemPipeline {
    pub(crate) fn build<'a>(
        registry: &SystemRegistry,
        systems: impl Iterator<Item = &'a UID>,
    ) -> Result<Self, RegistryError> {
        let mut codes = Vec::new();
        for uid in systems {
            let system = registry
                .get(uid)
                .ok_or(RegistryError::SystemDefinitionNotFound { uid: *uid })?;
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
                }
            }
        }
        Ok(())
    }
}
