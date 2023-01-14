use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct LifecycleComponent {
    pub alive: bool,
}

impl LifecycleComponent {
    
    pub fn alive() -> Self {
        Self { alive: true }
    }

    pub fn dead() -> Self {
        Self { alive: false }
    }
}