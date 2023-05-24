use mini3d::ui::controller::UIController;
use mini3d_derive::Component;

#[derive(Default, Component)]
pub struct OS {
    pub layout_active: bool,
    pub controller: UIController,
}