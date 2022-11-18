use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct CommandSignal {
    pub command: String,
}