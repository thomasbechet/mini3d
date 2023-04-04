use mini3d::{uid::UID, ui::controller::UIController, registry::component::Component};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct OS {
    pub layout_active: bool,
    pub controller: UIController,
}

impl Component for OS {}

impl OS {
    pub const NAME: &'static str = "os";
    pub const UID: UID = UID::new(Self::NAME);
}