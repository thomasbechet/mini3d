use std::collections::HashMap;

use anyhow::{Result, anyhow, Context};
use glam::{IVec2, UVec2};
use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::{color::Color, graphics::Graphics, SCREEN_RESOLUTION}, math::rect::IRect, context::input::InputContext};

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

    resolution: UVec2,
    background_color: Option<Color>,
}

impl Default for UI {
    fn default() -> Self {
        let mut ui = Self::new(SCREEN_RESOLUTION);

        let mut image = Sprite::new("alfred".into(), (50, 30).into(), (0, 0, 64, 64).into());
        image.set_color(Color::RED);
        ui.add_sprite("0", 5, UID::null(), image);

        let mut image = Sprite::new("alfred".into(), (60, 50).into(), (0, 0, 64, 64).into());
        image.set_color(Color::WHITE);
        ui.add_sprite("1", 10, UID::null(), image);

        let mut image = Sprite::new("alfred".into(), (70, 60).into(), (0, 0, 64, 64).into());
        image.set_color(Color::GREEN);
        ui.add_sprite("2", 6, UID::null(), image);

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

    pub fn new(resolution: UVec2) -> Self {
        Self { 
            widgets: Default::default(), 
            interaction_layout: Default::default(), 
            events: Default::default(), 
            interaction_events: Default::default(),
            resolution,
            background_color: Some(Color::BLACK),
        }
    }

    pub fn update(&mut self, input: &InputContext<'_>, time: f64) -> Result<()> {
        
        // Update interaction layout
        self.interaction_events.clear();
        let extent = IRect::new(0, 0, self.resolution.x, self.resolution.y);
        self.interaction_layout.update(input, extent, time, &mut self.interaction_events)?;

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

    pub fn render(&self, gfx: &mut Graphics, offset: IVec2, time: f64) {

        // Compute extent
        let extent = IRect::new(offset.x, offset.y, self.resolution.x, self.resolution.y);
        gfx.scissor(Some(extent));

        // Sort widgets before drawing
        let mut widgets = self.widgets.values().collect::<Vec<_>>();
        widgets.sort_by(|a, b| { a.z_index.cmp(&b.z_index) });

        // Background color
        if let Some(color) = self.background_color {
            gfx.fill_rect(extent, color);
        }

        // Draw widgets
        for widget in widgets {
            match &widget.variant {
                WidgetVariant::Label(label) => label.draw(gfx),
                WidgetVariant::Checkbox(checkbox) => checkbox.draw(gfx),
                WidgetVariant::Sprite(sprite) => sprite.draw(gfx),
                WidgetVariant::Viewport(viewport) => viewport.draw(gfx),
                _ => {}
            }
        }

        // Render interation layout
        self.interaction_layout.render(gfx, time);

        // Reset scissor
        gfx.scissor(None);
    }

    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.background_color = color;
    }

    /// Widgets API

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
        self.widgets.remove(&uid).with_context(|| "Widget not found")?;
        Ok(())
    }

    // pub fn events(&self) ->
}