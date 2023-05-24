use std::collections::HashMap;

use glam::{IVec2, UVec2};
use mini3d_derive::{Component, Error};

use crate::{ui::{widget::{layout::UILayout, Widget}, event::{UIEvent, EventContext, Event}, user::{UIUser, InteractionMode}}, ecs::{entity::Entity}, uid::UID, renderer::{color::Color, graphics::Graphics, SCREEN_VIEWPORT}, math::rect::IRect, feature::asset::ui_stylesheet::{UIStyleSheet, UIStyleSheetError}};

#[derive(Debug, Error)]
pub enum UIError {
    #[error("Duplicated user: {name}")]
    DuplicatedUser { name: String },
    #[error("User not found: {uid}")]
    UserNotFound { uid: UID },
    #[error("UIStyleSheetError: {0}")]
    UIStyleSheetError(UIStyleSheetError),
}

#[derive(Component)]
pub enum UIRenderTarget {
    Screen { offset: IVec2 },
    Canvas { offset: IVec2, canvas: Entity },
    Texture { offset: IVec2, texture: Entity },
}

impl Default for UIRenderTarget {
    fn default() -> Self {
        Self::Screen { offset: IVec2::ZERO }
    }
}

#[derive(Default, Component)]
#[component(name = "ui")]
pub struct UI {

    root: UILayout,
    users: HashMap<UID, UIUser>,
    stylesheet: UIStyleSheet,

    #[serialize(skip)]
    events: Vec<UIEvent>,

    resolution: UVec2,
    background_color: Option<Color>,
}

impl UI {

    pub fn new(resolution: UVec2, stylesheet: UIStyleSheet) -> Self {
        Self {
            root: UILayout::default(),
            users: Default::default(),
            stylesheet,
            events: Default::default(),
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
                            if skip_selection_move { continue; }
                            if user.mode != InteractionMode::Selection {
                                skip_selection_move = true;
                                user.mode = InteractionMode::Selection;
                                self.root.handle_event(&mut EventContext { user, events: &mut self.events, time }, &Event::ModeChanged);
                            }
                        },
                        Event::CursorMoved { .. } => {
                            if user.mode != InteractionMode::Cursor {
                                user.mode = InteractionMode::Cursor;
                                self.root.handle_event(&mut EventContext { user, events: &mut self.events, time }, &Event::ModeChanged);
                            }
                        },
                        _ => {}
                    }
                }

                // Dispatch event
                self.root.handle_event(&mut EventContext { user, events: &mut self.events, time }, &event);
            }
        }
        
        Ok(())
    }

    pub fn render(&self, gfx: &mut Graphics, offset: IVec2, time: f64) -> Result<(), UIError> {

        // Compute extent
        let extent = IRect::new(offset.x, offset.y, self.resolution.x, self.resolution.y).clamp(SCREEN_VIEWPORT);
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
        if self.users.contains_key(&uid) { return Err(UIError::DuplicatedUser { name: name.to_owned() }); }
        self.users.insert(uid, UIUser::new(name, IRect::new(0, 0, self.resolution.x, self.resolution.y)));
        Ok(uid)
    }

    pub fn remove_user(&mut self, uid: UID) -> Result<(), UIError> {
        self.users.remove(&uid).ok_or(UIError::UserNotFound { uid })?;
        Ok(())
    }

    pub fn user(&mut self, uid: UID) -> Result<&mut UIUser, UIError> {
        self.users.get_mut(&uid).ok_or(UIError::UserNotFound { uid })
    }

    pub fn add_styles(&mut self, stylesheet: &UIStyleSheet) -> Result<(), UIError> {
        self.stylesheet.merge(stylesheet).map_err(UIError::UIStyleSheetError)
    }
}