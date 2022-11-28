use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Serialize, Deserialize)]
pub struct LabelUI {
    text: String,
    font: UID,
}