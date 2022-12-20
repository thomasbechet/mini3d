use std::collections::{HashMap, HashSet};

use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};

use crate::{input::InputManager, uid::UID, math::rect::IRect, renderer::{backend::{CanvasHandle, RendererBackend, SceneCameraHandle}, RendererManager, RendererResourceManager, color::Color, SCREEN_RESOLUTION, SCREEN_WIDTH, SCREEN_HEIGHT}, asset::AssetManager};

use self::{interaction_layout::{InteractionLayout, InteractionEvent, InteractionInputs, AreaEvent}, button::Button, graphics::Graphics, label::Label, checkbox::Checkbox, sprite::Sprite, viewport::Viewport};

pub mod button;
pub mod graphics;
pub mod checkbox;
pub mod sprite;
pub mod label;
pub mod interaction_layout;
pub mod viewport;

#[derive(Serialize, Deserialize)]
pub enum Widget {
    Button(Button),
    Graphics(Graphics),
    Label(Label),
    Checkbox(Checkbox),
    Textbox,
    Viewport(Viewport),
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
            Widget::Viewport(viewport) => {},
            Widget::Sprite(sprite) => {},
        }
        Ok(())
    }

    fn release_renderer(&mut self, backend: &mut dyn RendererBackend) -> Result<()> {
        match self {
            Widget::Button(_) => todo!(),
            Widget::Graphics(_) => todo!(),
            Widget::Label(_) => todo!(),
            Widget::Checkbox(_) => todo!(),
            Widget::Textbox => todo!(),
            Widget::Viewport(viewport) => viewport.release_renderer(backend),
            Widget::Sprite(sprite) => sprite.release_renderer(backend),
        }
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
    #[serde(skip)]
    widget_removed: Vec<Widget>,

    width: u32,
    height: u32,
    background_color: Color,
    active: bool,

    #[serde(skip)]
    pub(crate) handle: Option<CanvasHandle>,
}

impl Default for UI {
    fn default() -> Self {
        let mut ui = Self::new(SCREEN_WIDTH, SCREEN_HEIGHT);

        let mut image = Sprite::new("alfred".into(), (50, 30).into(), (0, 0, 64, 64).into());
        image.set_color(Color::RED);
        image.set_z_index(5);
        ui.widgets.insert(0.into(), Widget::Sprite(image));

        let mut image = Sprite::new("alfred".into(), (60, 50).into(), (0, 0, 64, 64).into());
        image.set_color(Color::WHITE);
        image.set_z_index(10);
        ui.widgets.insert(1.into(), Widget::Sprite(image));

        let mut image = Sprite::new("alfred".into(), (70, 60).into(), (0, 0, 64, 64).into());
        image.set_color(Color::GREEN);
        image.set_z_index(6);
        ui.widgets.insert(2.into(), Widget::Sprite(image));

        let mut viewport = Viewport::new((0, 0).into(), SCREEN_RESOLUTION);
        viewport.set_z_index(2);
        // viewport.set_camera(camera)
        ui.widgets.insert("main_viewport".into(), Widget::Viewport(viewport));

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
        cameras: &HashMap<hecs::Entity, SceneCameraHandle>,
        asset: &AssetManager,
    ) -> Result<()> {

        // Create canvas
        if self.handle.is_none() {
            self.handle = Some(backend.canvas_add(self.width, self.height)?);
        }

        backend.canvas_set_clear_color(self.handle.unwrap(), self.background_color)?;

        for mut widget in self.widget_removed.drain(..) {
            widget.release_renderer(backend)?;
        }

        for widget in self.widgets.values_mut() {
            match widget {
                Widget::Button(button) => {},
                Widget::Graphics(paint) => {},
                Widget::Label(label) => {},
                Widget::Checkbox(checkbox) => {},
                Widget::Textbox => {},
                Widget::Viewport(viewport) => viewport.update_renderer(self.handle.unwrap(), cameras, backend)?,
                Widget::Sprite(sprite) => sprite.update_renderer(self.handle.unwrap(), resources, backend, asset)?,
            }
        }
        
        Ok(())
    }

    pub fn new(width: u32, height: u32) -> Self {
        Self { 
            widgets: Default::default(), 
            interaction_layout: Default::default(), 
            events: Default::default(), 
            interaction_events: Default::default(),
            widget_removed: Default::default(),
            width,
            height, 
            background_color: Color::BLACK, 
            active: true,
            handle: None 
        }
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

    pub fn add(&mut self, name: &str, widget: Widget) -> Result<UID> {
        let uid = UID::new(name);
        if self.widgets.contains_key(&uid) { return Err(anyhow!("Widget already exists")); }        
        self.widgets.insert(uid, widget);
        Ok(uid)
    }

    pub fn remove(&mut self, uid: UID) -> Result<()> {
        self.widget_removed.push(self.widgets.remove(&uid).with_context(|| "Widget not found")?);
        Ok(())
    }

    pub fn get(&self, uid: UID) -> Result<&Widget> {
        self.widgets.get(&uid).with_context(|| "Widget not found")
    }

    pub fn get_mut(&mut self, uid: UID) -> Result<&mut Widget> {
        self.widgets.get_mut(&uid).with_context(|| "Widget not found")
    }

    // pub fn events(&self) ->
}