use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RhaiScript {
    pub source: String,
}