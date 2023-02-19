use glam::IVec2;
use serde::{Serialize, Deserialize};

use crate::{ui::UI, ecs::entity::Entity, uid::UID, registry::component::Component};

#[derive(Serialize, Deserialize)]
pub enum UIRenderTarget {
    Screen { offset: IVec2 },
    Canvas { offset: IVec2, canvas: Entity },
    Texture { offset: IVec2, texture: Entity },
}

#[derive(Serialize, Deserialize)]
pub struct UIComponent {
    pub ui: UI,
    pub render_targets: Vec<UIRenderTarget>,
    pub visible: bool,
    pub active: bool,
}

impl Component for UIComponent {}

impl UIComponent {

    // TODO: find better name
    pub const NAME: &'static str = "ui_component";
    pub const UID: UID = UIComponent::NAME.into();

    pub fn new(ui: UI, render_target: UIRenderTarget) -> Self {
        Self {
            ui,
            render_targets: vec![render_target],
            visible: true,
            active: true,
        }
    }

    pub fn add_render_target(&mut self, render_target: UIRenderTarget) {
        self.render_targets.push(render_target);
    }
}