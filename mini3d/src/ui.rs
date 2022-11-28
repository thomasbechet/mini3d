use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::graphics::command_buffer::CommandBuffer;

use self::{navigation_layout::NavigationLayout, button::ButtonUI, label::LabelUI, viewport::ViewportUI};

pub mod button;
pub mod label;
pub mod navigation_layout;
pub mod viewport;

#[derive(Serialize, Deserialize)]
enum Widget {
    Button(ButtonUI),
    Label(LabelUI),
    Viewport(ViewportUI),
}

#[derive(Serialize, Deserialize)]
pub struct UI {
    navigation_layout: NavigationLayout,
    root: Widget,
}

impl UI {
    pub fn render(&self) -> Result<CommandBuffer> {
        Ok(CommandBuffer::empty())
    }
}