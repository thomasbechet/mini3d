use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Serialize, Deserialize)]
pub struct ModelComponent {
    pub model: UID,
    #[serde(skip)]
    pub handle: UID,
}

impl ModelComponent {
    pub fn new(model: UID) -> Self {
        Self { model, handle: UID::null() }
    }
}