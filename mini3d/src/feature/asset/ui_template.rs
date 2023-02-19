use serde::{Serialize, Deserialize};

use crate::{registry::asset::Asset, uid::UID};

#[derive(Clone, Serialize, Deserialize)]
pub struct UITemplate {
    
}

impl Asset for UITemplate {}

impl UITemplate {
    pub const NAME: &'static str = "ui_template";
    pub const UID: UID = UID::new(UITemplate::NAME);
}