use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum SystemEvent {
    Shutdown,
}