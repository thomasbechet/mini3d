use serde::{Serialize, Deserialize};

use super::Component;

#[derive(Serialize, Deserialize)]
pub struct LifecycleComponent {
    pub alive: bool,
}

impl Default for LifecycleComponent {
    fn default() -> Self {
        Self { alive: true }
    }
}

impl Component for LifecycleComponent {}