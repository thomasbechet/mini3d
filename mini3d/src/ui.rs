use std::collections::{HashMap, HashSet};

use anyhow::Result;
use serde::{Serialize, Deserialize};

use crate::{input::InputManager, uid::UID, math::rect::IRect, renderer::{backend::{CanvasHandle, RendererBackend}, RendererManager, RendererResourceManager, color::Color}, asset::AssetManager};

use self::{interaction_layout::{InteractionLayout, InteractionEvent, InteractionInputs, AreaEvent}, button::Button, graphics::Graphics, label::Label, checkbox::Checkbox, sprite::Sprite};

pub mod button;
pub mod graphics;
pub mod checkbox;
pub mod sprite;
pub mod label;
pub mod interaction_layout;
pub mod viewport;

#[derive(Serialize, Deserialize)]
enum Widget {
    Button(Button),
    Graphics(Graphics),
    Label(Label),
    Checkbox(Checkbox),
    Textbox,
    Viewport,
    Sprite(Sprite),
}

impl Widget {
    fn handle_event(&mut self, event: &AreaEvent, interaction_layout: &mut InteractionLayout) -> Result<()> {
        match self {
            Widget::Button(button) => {},
            Widget::Graphics(paint) => {},
            Widget::Label(label) => {},
            Widget::Checkbox(checkbox) => {},
            Widget::Textbox => {},
            Widget::Viewport => {},
            Widget::Sprite(sprite) => {},
        }
        Ok(())
    }
}

struct HandleContext {
    events: Vec<UIEvent>,
}

pub enum UIEvent {
    ButtonClicked {
        button: UID,
        profile: UID,
    },
    CheckboxChanged {
        checkbox: UID,
        profile: UID,
        checked: bool,
    }
}

#[derive(Serialize, Deserialize)]
pub struct UI {

    widgets: HashMap<UID, Widget>,
    interaction_layout: InteractionLayout,

    #[serde(skip)]
    events: Vec<UIEvent>,
    #[serde(skip)]
    interaction_events: Vec<InteractionEvent>,

    width: u32,
    height: u32,
    background_color: Color,
    active: bool,

    #[serde(skip)]
    pub(crate) handle: Option<CanvasHandle>,
}

impl Default for UI {
    fn default() -> Self {
        let mut ui = Self { 
            widgets: Default::default(), 
            interaction_layout: Default::default(),
            events: Default::default(),
            interaction_events: Default::default(),
            width: 300,
            height: 300,
            background_color: Color::BLACK,
            active: Default::default(),
            handle: None,
        };

        let mut image = Sprite::new("alfred".into(), (50, 30).into(), (0, 0, 64, 64).into());
        image.set_color(Color::RED);
        image.set_z_index(5);
        ui.widgets.insert(0.into(), Widget::Sprite(image));

        let mut image = Sprite::new("alfred".into(), (60, 50).into(), (0, 0, 64, 64).into());
        image.set_color(Color::WHITE);
        ui.widgets.insert(1.into(), Widget::Sprite(image));

        let mut image = Sprite::new("alfred".into(), (70, 60).into(), (0, 0, 64, 64).into());
        image.set_color(Color::WHITE);
        image.set_z_index(2);
        ui.widgets.insert(2.into(), Widget::Sprite(image));

        // ui.interaction_layout.add_area(0.into(), IRect::new(5, 5, 100, 50)).unwrap();
        // ui.interaction_layout.add_area(1.into(), IRect::new(5, 200, 100, 50)).unwrap();
        // ui.interaction_layout.add_area(2.into(), IRect::new(150, 5, 100, 50)).unwrap();
        ui.interaction_layout.add_area(3.into(), IRect::new(150, 200, 50, 50)).unwrap();
        ui.interaction_layout.add_area(4.into(), IRect::new(400, 50, 100, 200)).unwrap();

        ui.interaction_layout.add_area(5.into(), IRect::new(0, 0, 10, 10)).unwrap();
        ui.interaction_layout.add_area(6.into(), IRect::new(10, 0, 10, 10)).unwrap();
        ui.interaction_layout.add_area(7.into(), IRect::new(20, 0, 10, 10)).unwrap();
        ui.interaction_layout.add_area(8.into(), IRect::new(30, 0, 10, 10)).unwrap();
        ui.interaction_layout.add_area(9.into(), IRect::new(40, 0, 10, 10)).unwrap();
        ui
    }
}

impl UI {

    pub(crate) fn update_renderer(
        &mut self, 
        backend: &mut impl RendererBackend,
        resources: &mut RendererResourceManager,
        asset: &AssetManager,
    ) -> Result<()> {

        // Create canvas
        if self.handle.is_none() {
            self.handle = Some(backend.canvas_add(self.width, self.height)?);
        }

        backend.canvas_set_clear_color(self.handle.unwrap(), self.background_color)?;

        for widget in self.widgets.values_mut() {
            match widget {
                Widget::Button(button) => {},
                Widget::Graphics(paint) => {},
                Widget::Label(label) => {},
                Widget::Checkbox(checkbox) => {},
                Widget::Textbox => {},
                Widget::Viewport => {},
                Widget::Sprite(sprite) => sprite.update_renderer(self.handle.unwrap(), resources, backend, asset)?,
            }
        }
        
        Ok(())
    }

    pub(crate) fn release_renderer(
        &mut self,
        renderer: &mut RendererManager,
    ) -> Result<()> {
        if let Some(handle) = self.handle {
            renderer.canvases_removed.insert(handle);
        }
        Ok(())
    }

    pub fn update(&mut self, input: &InputManager, time: f64) -> Result<()> {
        
        // Update interaction layout
        self.interaction_events.clear();
        self.interaction_layout.update(input, time, &mut self.interaction_events)?;

        // Dispatch events
        for event in &self.interaction_events {
            match event {
                InteractionEvent::Area { area, event } => {
                    // self.widgets.get(area).unwrap();
                    // instance.widget.
                },
                InteractionEvent::Profile { profile, event } => {

                },
            }
        }

        // self.navigation_layout.hovered_area(profile)
        Ok(())
    }

    pub fn add_profile(&mut self, name: &str, inputs: InteractionInputs) -> Result<UID> {
        self.interaction_layout.add_profile(name, inputs)
    }

    // pub fn add<W: Widget>()

    // pub fn events(&self) ->
}