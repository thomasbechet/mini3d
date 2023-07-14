use mini3d_derive::{Component, Reflect, Serialize};

use crate::uid::UID;

#[derive(Debug, Default, Serialize, Reflect, Component, Clone)]
pub struct SystemGraphEntry {
    system: UID,
    group: Option<String>,
    dependencies: Vec<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SystemGraph {
    entries: Vec<SystemGraphEntry>,
}

impl SystemGraph {
    pub fn single(system: UID, group: Option<String>) -> Self {
        Self {
            entries: vec![SystemGraphEntry {
                system,
                group,
                dependencies: Vec::new(),
            }],
        }
    }

    pub fn linear(systems: &[(UID, Option<String>)]) -> Self {
        Self {
            entries: systems
                .iter()
                .enumerate()
                .map(|(i, (system, group))| SystemGraphEntry {
                    system: *system,
                    group: group.clone(),
                    dependencies: if i == 0 {
                        Vec::new()
                    } else {
                        vec![i as u32 - 1]
                    },
                })
                .collect::<Vec<_>>(),
        }
    }
}
