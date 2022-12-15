use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Serialize, Deserialize)]
pub struct Label {
    text: String,
    font: UID,
}