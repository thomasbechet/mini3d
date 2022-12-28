use std::collections::HashMap;

use anyhow::{Result, anyhow, Context};
use serde::{Serialize, Deserialize};

use crate::{input::InputManager, uid::UID, math::rect::IRect, renderer::{backend::{CanvasHandle, RendererBackend, SceneCameraHandle}, RendererResourceManager, color::Color, SCREEN_RESOLUTION, SCREEN_WIDTH, SCREEN_HEIGHT}, asset::AssetManager};

use self::{interaction_layout::{InteractionLayout, InteractionEvent, InteractionInputs}, button::Button, label::Label, checkbox::Checkbox, sprite::Sprite, viewport::Viewport};

pub mod button;
pub mod graphics;
pub mod checkbox;
pub mod sprite;
pub mod label;
pub mod interaction_layout;
pub mod viewport;

macro_rules! define_add {
    ($name:ident, $fname:ident, $widget:ident) => {
        pub fn $fname(&mut self, name: &str, z_index: i32, parent: UID, $name: $widget) -> Result<UID> {
            let uid = UID::new(name);
            if self.widgets.contains_key(&uid) { return Err(anyhow!("Widget already exists")); }
            self.widgets.insert(uid, Widget { z_index, parent, variant: WidgetVariant::$widget($name) });
            Ok(uid)
        }
    };
}

macro_rules! define_get {
    ($name:ident, $fname:ident, $widget:ident) => {
        pub fn $fname(&self, uid: UID) -> Result<&$widget> {
            let widget = self.widgets.get(&uid).with_context(|| "Widget not found")?;
            match &widget.variant {
                WidgetVariant::$widget(widget) => Ok(widget),
                _ => { Err(anyhow!("Widget is not a {}", stringify!($widget))) }
            }
        }
    };
}

macro_rules! define_get_mut {
    ($name:ident, $fname:ident, $widget:ident) => {
        pub fn $fname(&mut self, uid: UID) -> Result<&mut $widget> {
            let widget = self.widgets.get_mut(&uid).with_context(|| "Widget not found")?;
            match &mut widget.variant {
                WidgetVariant::$widget(widget) => Ok(widget),
                _ => { Err(anyhow!("Widget is not a {}", stringify!($widget))) }
            }
        }
    };
}

// impl Widget {

//     fn handle_event(&mut self, event: &AreaEvent, interaction_layout: &mut InteractionLayout) -> Result<()> {
//         match self {
//             Widget::Button(button) => {},
//             Widget::Graphics(paint) => {},
//             Widget::Label(label) => {},
//             Widget::Checkbox(checkbox) => {},
//             Widget::Textbox => {},
//             Widget::Viewport(viewport) => {},
//             Widget::Sprite(sprite) => {},
//         }
//         Ok(())
//     }

//     fn release_backend(&mut self, backend: &mut dyn RendererBackend) -> Result<()> {
//         if let Widget::Viewport(viewport) = self {
//             viewport.release_backend(backend)?;
//         }
//         Ok(())
//     }
// }

#[derive(Serialize, Deserialize)]
enum WidgetVariant {
    Button(Button),
    Checkbox(Checkbox),
    Label(Label),
    Sprite(Sprite),
    Viewport(Viewport),
}

#[derive(Serialize, Deserialize)]
struct Widget {
    z_index: i32,
    parent: UID,
    variant: WidgetVariant,
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
    viewports_removed: Vec<Viewport>,

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
        ui.add_sprite("0", 5, UID::null(), image);

        let mut image = Sprite::new("alfred".into(), (60, 50).into(), (0, 0, 64, 64).into());
        image.set_color(Color::WHITE);
        ui.add_sprite("1", 10, UID::null(), image);

        let mut image = Sprite::new("alfred".into(), (70, 60).into(), (0, 0, 64, 64).into());
        image.set_color(Color::GREEN);
        ui.add_sprite("2", 6, UID::null(), image);

        let mut viewport = Viewport::new((0, 0).into(), SCREEN_RESOLUTION);
        ui.add_viewport("main_viewport", 2, UID::null(), viewport);
        // viewport.set_camera(camera)
        
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

    pub(crate) fn update_backend(
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

        // Release resources for removed viewports
        for mut viewport in self.viewports_removed.drain(..) {
            viewport.release_backend(backend)?;
        }

        // Sort widgets before drawing
        let mut widgets = self.widgets.values_mut().collect::<Vec<_>>();
        widgets.sort_by(|a, b| { a.z_index.cmp(&b.z_index) });

        // Draw widgets
        backend.canvas_begin(self.handle.unwrap(), self.background_color)?;
        for widget in widgets {
            match &mut widget.variant {
                WidgetVariant::Label(label) => label.draw(resources, backend, asset)?,
                WidgetVariant::Checkbox(checkbox) => checkbox.draw(backend)?,
                WidgetVariant::Sprite(sprite) => sprite.draw(resources, backend, asset)?,
                WidgetVariant::Viewport(viewport) => viewport.draw(cameras, backend)?,
                _ => {}
            }
        }
        backend.canvas_end()?;
        
        Ok(())
    }

    pub fn new(width: u32, height: u32) -> Self {
        Self { 
            widgets: Default::default(), 
            interaction_layout: Default::default(), 
            events: Default::default(), 
            interaction_events: Default::default(),
            viewports_removed: Default::default(),
            width,
            height, 
            background_color: Color::TRANSPARENT, 
            active: true,
            handle: None,
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

    define_add!(label, add_label, Label);
    define_get!(label, label, Label);
    define_get_mut!(label, label_mut, Label);

    define_add!(sprite, add_sprite, Sprite);
    define_get!(sprite, sprite, Sprite);
    define_get_mut!(sprite, sprite_mut, Sprite);

    define_add!(checkbox, add_checkbox, Checkbox);
    define_get!(checkbox, checkbox, Checkbox);
    define_get_mut!(checkbox, checkbox_mut, Checkbox);

    define_add!(viewport, add_viewport, Viewport);
    define_get!(viewport, viewport, Viewport);
    define_get_mut!(viewport, viewport_mut, Viewport);

    pub fn remove(&mut self, uid: UID) -> Result<()> {
        let widget = self.widgets.remove(&uid).with_context(|| "Widget not found")?;
        if let WidgetVariant::Viewport(viewport) = widget.variant {
            self.viewports_removed.push(viewport);
        }
        Ok(())
    }

    // pub fn events(&self) ->
}