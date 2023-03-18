use std::collections::HashMap;

use anyhow::{Result, anyhow};
use glam::{IVec2, UVec2};
use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::{color::Color, graphics::Graphics, SCREEN_RESOLUTION}, math::rect::IRect, context::input::InputContext};

use self::{profile::{Profile, ProfileInputs}, event::{UIEvent, Event, EventContext}, widget::{Widget, layout::Layout}};

pub mod event;
pub mod profile;
pub mod widget;

// macro_rules! define_add {
//     ($name:ident, $fname:ident, $widget:ident) => {
//         pub fn $fname(&mut self, name: &str, z_index: i32, parent: UID, $name: $widget) -> Result<UID> {
//             let uid = UID::new(name);
//             if self.widgets.contains_key(&uid) { return Err(anyhow!("Widget already exists")); }
//             self.widgets.insert(uid, Widget { z_index, parent, variant: WidgetVariant::$widget($name) });
//             Ok(uid)
//         }
//     };
// }

// macro_rules! define_get {
//     ($name:ident, $fname:ident, $widget:ident) => {
//         pub fn $fname(&self, uid: UID) -> Result<&$widget> {
//             let widget = self.widgets.get(&uid).with_context(|| "Widget not found")?;
//             match &widget.variant {
//                 WidgetVariant::$widget(widget) => Ok(widget),
//                 _ => { Err(anyhow!("Widget is not a {}", stringify!($widget))) }
//             }
//         }
//     };
// }

// macro_rules! define_get_mut {
//     ($name:ident, $fname:ident, $widget:ident) => {
//         pub fn $fname(&mut self, uid: UID) -> Result<&mut $widget> {
//             let widget = self.widgets.get_mut(&uid).with_context(|| "Widget not found")?;
//             match &mut widget.variant {
//                 WidgetVariant::$widget(widget) => Ok(widget),
//                 _ => { Err(anyhow!("Widget is not a {}", stringify!($widget))) }
//             }
//         }
//     };
// }

#[derive(Serialize, Deserialize)]
pub struct UI {

    root: Layout,
    profiles: HashMap<UID, Profile>,

    #[serde(skip)]
    events: Vec<UIEvent>,

    resolution: UVec2,
    background_color: Option<Color>,
}

impl UI {

    pub fn new(resolution: UVec2) -> Self {
        Self {
            root: Layout::default(),
            profiles: Default::default(),
            events: Default::default(),
            resolution,
            background_color: Some(Color::BLACK),
        }
    }

    pub fn update(&mut self, input: &InputContext<'_>, time: f64) -> Result<()> {
        
        let mut events: Vec<Event> = Default::default();

        // Update profiles
        for profile in self.profiles.values_mut() {
            
            // Clear events
            events.clear();

            // Update profile
            profile.update(input, &mut events)?;

            // Distpach events
            for event in &events {
                self.root.handle_event(&mut EventContext { profile, events: &mut self.events, time }, event);
            }
        }
        
        Ok(())
    }

    pub fn render(&self, gfx: &mut Graphics, offset: IVec2, time: f64) {

        // Compute extent
        let extent = IRect::new(offset.x, offset.y, self.resolution.x, self.resolution.y);
        gfx.scissor(Some(extent));

        // Background color
        if let Some(color) = self.background_color {
            gfx.fill_rect(extent, color);
        }

        // Render
        self.root.render(gfx, time);

        // Render profiles
        for profile in self.profiles.values() {
            profile.render(gfx, time);
        }

        // Reset scissor
        gfx.scissor(None);
    }

    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.background_color = color;
    }
  
    pub fn add_profile(&mut self, name: &str, inputs: ProfileInputs) -> Result<UID> {
        let uid = UID::new(name);
        if self.profiles.contains_key(&uid) { return Err(anyhow!("Profile name already exists")); }
        self.profiles.insert(uid, Profile::new(name, inputs));
        Ok(uid)
    }

    pub fn root(&mut self) -> &'_ mut Layout {
        &mut self.root
    }
}