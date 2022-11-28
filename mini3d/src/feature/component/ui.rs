use serde::{Serialize, Deserialize};

use crate::ui::UI;

#[derive(Serialize, Deserialize)]
pub struct UIComponent {
    pub ui: UI,
}