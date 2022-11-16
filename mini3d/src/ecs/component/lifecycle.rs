use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct LifecycleComponent {
    pub alive: bool,
}

impl Default for LifecycleComponent {
    fn default() -> Self {
        Self { alive: true }
    }
}