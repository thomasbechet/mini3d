use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::uid::UID;

#[derive(Serialize, Deserialize)]
struct Navigation {
    up: Option<UID>,
    down: Option<UID>,
    left: Option<UID>,
    right: Option<UID>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Layout {
    sorted_widgets: Vec<UID>,
    navigations: HashMap<UID, Navigation>,
}