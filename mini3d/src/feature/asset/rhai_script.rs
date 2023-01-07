use serde::{Serialize, Deserialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct RhaiScriptAsset {
    pub source: String,
}