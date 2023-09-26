use mini3d_derive::{Component, Reflect, Serialize};

#[derive(Default, Component, Serialize, Reflect)]
pub struct OS {
    pub layout_active: bool,
    // pub controller: UIController,
}
