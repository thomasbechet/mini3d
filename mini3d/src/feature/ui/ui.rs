use std::collections::HashMap;

use glam::{IVec2, UVec2};
use mini3d_derive::{Component, Error, Reflect, Serialize};

use crate::{
    ecs::entity::Entity,
    math::rect::IRect,
    renderer::{color::Color, graphics::Graphics, SCREEN_VIEWPORT},
    utils::uid::UID,
};

use super::ui_stylesheet::{UIStyleSheet, UIStyleSheetError};

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

#[derive(Debug, Error)]
pub enum UIError {
    #[error("Duplicated user: {name}")]
    DuplicatedUser { name: String },
    #[error("User not found: {uid}")]
    UserNotFound { uid: UID },
    #[error("UIStyleSheetError: {0}")]
    UIStyleSheetError(UIStyleSheetError),
}

#[derive(Component, Serialize, Reflect, Clone)]
pub enum UIRenderTarget {
    Screen { offset: IVec2 },
    Canvas { offset: IVec2, canvas: Entity },
    Texture { offset: IVec2, texture: Entity },
}

impl Default for UIRenderTarget {
    fn default() -> Self {
        Self::Screen {
            offset: IVec2::ZERO,
        }
    }
}

#[derive(Default, Component, Serialize, Reflect)]
pub struct UI {
    root: Entity,
    users: HashMap<UID, UIUser>,
    stylesheet: UIStyleSheet,
    resolution: UVec2,
    background_color: Option<Color>,
}

impl UI {
    pub fn new(resolution: UVec2, stylesheet: UIStyleSheet) -> Self {
        Self {
            root: Default::default(),
            users: Default::default(),
            stylesheet,
            resolution,
            background_color: Some(Color::BLACK),
        }
    }

    pub fn update(&mut self, time: f64) -> Result<(), UIError> {
        // Clear events
        self.events.clear();

        // Update profiles
        for user in self.users.values_mut() {
            let mut skip_selection_move = false;

            // Dispatch events
            for event in user.events.drain(..).collect::<Vec<_>>() {
                // Generate change mode event
                if !user.locked {
                    match event {
                        Event::SelectionMoved { .. } => {
                            if skip_selection_move {
                                continue;
                            }
                            if user.mode != InteractionMode::Selection {
                                skip_selection_move = true;
                                user.mode = InteractionMode::Selection;
                                self.root.handle_event(
                                    &mut EventContext {
                                        user,
                                        events: &mut self.events,
                                        time,
                                    },
                                    &Event::ModeChanged,
                                );
                            }
                        }
                        Event::CursorMoved { .. } => {
                            if user.mode != InteractionMode::Cursor {
                                user.mode = InteractionMode::Cursor;
                                self.root.handle_event(
                                    &mut EventContext {
                                        user,
                                        events: &mut self.events,
                                        time,
                                    },
                                    &Event::ModeChanged,
                                );
                            }
                        }
                        _ => {}
                    }
                }

                // Dispatch event
                self.root.handle_event(
                    &mut EventContext {
                        user,
                        events: &mut self.events,
                        time,
                    },
                    &event,
                );
            }
        }

        Ok(())
    }

    pub fn render(&self, gfx: &mut Graphics, offset: IVec2, time: f64) -> Result<(), UIError> {
        // Compute extent
        let extent = IRect::new(offset.x, offset.y, self.resolution.x, self.resolution.y)
            .clamp(SCREEN_VIEWPORT);
        gfx.scissor(Some(extent));

        // Background color
        if let Some(color) = self.background_color {
            gfx.fill_rect(extent, color);
        }

        // Render
        self.root.render(gfx, &self.stylesheet, offset, time);

        // Render profiles
        for user in self.users.values() {
            user.render(gfx, time);
        }

        // Reset scissor
        gfx.scissor(None);

        Ok(())
    }

    pub fn set_background_color(&mut self, color: Option<Color>) {
        self.background_color = color;
    }

    pub fn root(&mut self) -> &'_ mut UILayout {
        &mut self.root
    }

    pub fn events(&self) -> &'_ [UIEvent] {
        &self.events
    }

    pub fn add_user(&mut self, name: &str) -> Result<UID, UIError> {
        let uid = UID::new(name);
        if self.users.contains_key(&uid) {
            return Err(UIError::DuplicatedUser {
                name: name.to_owned(),
            });
        }
        self.users.insert(
            uid,
            UIUser::new(name, IRect::new(0, 0, self.resolution.x, self.resolution.y)),
        );
        Ok(uid)
    }

    pub fn remove_user(&mut self, uid: UID) -> Result<(), UIError> {
        self.users
            .remove(&uid)
            .ok_or(UIError::UserNotFound { uid })?;
        Ok(())
    }

    pub fn user(&mut self, uid: UID) -> Result<&mut UIUser, UIError> {
        self.users
            .get_mut(&uid)
            .ok_or(UIError::UserNotFound { uid })
    }

    pub fn add_styles(&mut self, stylesheet: &UIStyleSheet) -> Result<(), UIError> {
        self.stylesheet
            .merge(stylesheet)
            .map_err(UIError::UIStyleSheetError)
    }
}