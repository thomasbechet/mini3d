use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Checkbox {
    checked: bool,
}