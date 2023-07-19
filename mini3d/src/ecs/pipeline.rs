use crate::registry::system::SystemId;

struct PipelineStep {
    system: SystemId,
    next: u32,
}

pub(crate) struct SystemPipeline {
    steps: Vec<PipelineStep>,
}

impl SystemPipeline {
    // pub(crate) fn build<'a>(
    //     registry: &SystemRegistry,
    //     systems: impl Iterator<Item = &'a UID>,
    // ) -> Result<Self, RegistryError> {
    //     let mut codes = Vec::new();
    //     for uid in systems {
    //         let system = registry
    //             .find(uid)
    //             .ok_or(RegistryError::SystemDefinitionNotFound { uid: *uid })?;
    //         codes.push(system.code);
    //     }
    //     Ok(Self { systems: codes })
    // }

    // pub(crate) fn run(
    //     &self,
    //     context: &mut ExclusiveSystemContext,
    //     _script: &ScriptManager,
    // ) -> SystemResult {
    //     for system in &self.systems {
    //         match system {
    //             SystemCode::Static(callback) => callback(context)?,
    //             SystemCode::Script(_uid) => {
    //                 todo!()
    //             }
    //         }
    //     }
    //     Ok(())
    // }
}
