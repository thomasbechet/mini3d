use std::collections::HashMap;

use glam::IVec2;
use mini3d_derive::{Error, Serialize};

use crate::{
    feature::component::ui::ui_stylesheet::UIStyleSheet,
    math::rect::IRect,
    renderer::{graphics::Graphics, SCREEN_VIEWPORT},
    ui::{
        event::{Direction, Event, EventContext},
        user::InteractionMode,
    },
    uid::UID,
};

use super::{
    button::UIButton, checkbox::UICheckBox, label::UILabel, slider::UISlider, sprite::UISprite,
    textbox::UITextBox, viewport::UIViewport, Widget,
};

#[derive(Serialize)]
pub(crate) enum WidgetVariant {
    Button(UIButton),
    Checkbox(UICheckBox),
    Label(UILabel),
    Slider(UISlider),
    Sprite(UISprite),
    TextBox(UITextBox),
    Viewport(UIViewport),
}

impl Widget for WidgetVariant {
    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool {
        match self {
            WidgetVariant::Button(button) => button.handle_event(ctx, event),
            WidgetVariant::Checkbox(checkbox) => checkbox.handle_event(ctx, event),
            WidgetVariant::Label(label) => false,
            WidgetVariant::Slider(slider) => false,
            WidgetVariant::TextBox(textbox) => textbox.handle_event(ctx, event),
            _ => false,
        }
    }

    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, time: f64) {
        match self {
            WidgetVariant::Button(button) => button.render(gfx, styles, offset, time),
            WidgetVariant::Checkbox(checkbox) => checkbox.render(gfx, styles, offset, time),
            WidgetVariant::Label(label) => label.render(gfx, styles, offset, time),
            WidgetVariant::Slider(slider) => {}
            WidgetVariant::Sprite(sprite) => sprite.render(gfx, styles, offset, time),
            WidgetVariant::TextBox(textbox) => textbox.render(gfx, styles, offset, time),
            WidgetVariant::Viewport(viewport) => viewport.render(gfx, styles, offset, time),
        }
    }

    fn extent(&self) -> IRect {
        match self {
            WidgetVariant::Button(button) => button.extent(),
            WidgetVariant::Checkbox(checkbox) => checkbox.extent(),
            WidgetVariant::Label(label) => IRect::new(0, 0, 0, 0),
            WidgetVariant::Slider(slider) => IRect::new(0, 0, 0, 0),
            WidgetVariant::Sprite(sprite) => sprite.extent(),
            WidgetVariant::TextBox(textbox) => textbox.extent(),
            WidgetVariant::Viewport(viewport) => viewport.extent(),
        }
    }

    fn is_focusable(&self) -> bool {
        match self {
            WidgetVariant::Button(button) => button.is_focusable(),
            WidgetVariant::Checkbox(checkbox) => checkbox.is_focusable(),
            WidgetVariant::Label(label) => false,
            WidgetVariant::Slider(slider) => false,
            WidgetVariant::Sprite(sprite) => false,
            WidgetVariant::TextBox(textbox) => textbox.is_focusable(),
            WidgetVariant::Viewport(viewport) => false,
        }
    }

    fn is_selectable(&self) -> bool {
        true
    }
}

#[derive(Default, Serialize)]
pub struct Navigation {
    pub up: Option<UID>,
    pub down: Option<UID>,
    pub left: Option<UID>,
    pub right: Option<UID>,
}

impl Navigation {
    fn get(&self, direction: Direction) -> Option<UID> {
        match direction {
            Direction::Up => self.up,
            Direction::Down => self.down,
            Direction::Left => self.left,
            Direction::Right => self.right,
        }
    }
}

#[derive(Serialize)]
struct WidgetEntry {
    name: String,
    z_index: i32,
    navigation: Navigation,
    widget: WidgetVariant,
}

#[derive(Debug, Error)]
pub enum UILayoutError {
    #[error("Widget not found: {uid}")]
    WidgetNotFound { uid: UID },
}

#[derive(Default, Serialize)]
pub struct UILayout {
    widgets: HashMap<UID, WidgetEntry>,
    default_target: Option<UID>,
    profiles_focus: HashMap<UID, UID>,
    profiles_target: HashMap<UID, UID>,
}

impl UILayout {
    fn add(&mut self, uid: UID, entry: WidgetEntry) {
        if self.default_target.is_none() {
            self.default_target = Some(uid);
        }
        self.widgets.insert(uid, entry);
    }

    pub fn add_button(&mut self, name: &str, z_index: i32, button: UIButton) -> UID {
        let uid: UID = name.into();
        self.add(
            uid,
            WidgetEntry {
                name: name.to_string(),
                z_index,
                navigation: Navigation::default(),
                widget: WidgetVariant::Button(button),
            },
        );
        uid
    }

    pub fn add_sprite(&mut self, name: &str, z_index: i32, sprite: UISprite) -> UID {
        let uid: UID = name.into();
        self.add(
            uid,
            WidgetEntry {
                name: name.to_string(),
                z_index,
                navigation: Navigation::default(),
                widget: WidgetVariant::Sprite(sprite),
            },
        );
        uid
    }

    pub fn add_textbox(&mut self, name: &str, z_index: i32, textbox: UITextBox) -> UID {
        let uid: UID = name.into();
        self.add(
            uid,
            WidgetEntry {
                name: name.to_string(),
                z_index,
                navigation: Navigation::default(),
                widget: WidgetVariant::TextBox(textbox),
            },
        );
        uid
    }

    pub fn add_checkbox(&mut self, name: &str, z_index: i32, checkbox: UICheckBox) -> UID {
        let uid: UID = name.into();
        self.add(
            uid,
            WidgetEntry {
                name: name.to_string(),
                z_index,
                navigation: Navigation::default(),
                widget: WidgetVariant::Checkbox(checkbox),
            },
        );
        uid
    }

    pub fn add_label(&mut self, name: &str, z_index: i32, label: UILabel) -> UID {
        let uid: UID = name.into();
        self.add(
            uid,
            WidgetEntry {
                name: name.to_string(),
                z_index,
                navigation: Navigation::default(),
                widget: WidgetVariant::Label(label),
            },
        );
        uid
    }

    pub fn add_viewport(&mut self, name: &str, z_index: i32, viewport: UIViewport) -> UID {
        let uid: UID = name.into();
        self.add(
            uid,
            WidgetEntry {
                name: name.to_string(),
                z_index,
                navigation: Navigation::default(),
                widget: WidgetVariant::Viewport(viewport),
            },
        );
        uid
    }

    pub fn set_navigation(
        &mut self,
        widget: UID,
        navigation: Navigation,
    ) -> Result<(), UILayoutError> {
        self.widgets
            .get_mut(&widget)
            .ok_or(UILayoutError::WidgetNotFound { uid: widget })?
            .navigation = navigation;
        Ok(())
    }
}

impl Widget for UILayout {
    fn handle_event(&mut self, ctx: &mut EventContext, event: &Event) -> bool {
        let profile_uid = ctx.user.uid();

        match event {
            Event::PrimaryJustPressed => {
                if let Some(target) = self.profiles_target.get(&profile_uid) {
                    let current_focus = self.profiles_focus.get(&profile_uid).copied();

                    {
                        // Unfocus
                        if let Some(current_focus) = current_focus {
                            if current_focus != *target {
                                let entry = self.widgets.get_mut(&current_focus).unwrap();
                                entry.widget.handle_event(ctx, &Event::LooseFocus);
                                self.profiles_focus.remove(&profile_uid);
                            }
                        }
                    }

                    {
                        // Focus
                        if current_focus != Some(*target) {
                            let entry = self.widgets.get_mut(target).unwrap();
                            if entry.widget.is_focusable() {
                                entry.widget.handle_event(ctx, &Event::GainFocus);
                                self.profiles_focus.insert(profile_uid, *target);
                            }
                        }
                    }

                    {
                        // Just pressed
                        let entry = self.widgets.get_mut(target).unwrap();
                        entry.widget.handle_event(ctx, &Event::PrimaryJustPressed);
                    }
                } else if let Some(focus) = self.profiles_focus.get(&profile_uid) {
                    let entry = self.widgets.get_mut(focus).unwrap();
                    entry.widget.handle_event(ctx, &Event::LooseFocus);
                    self.profiles_focus.remove(&profile_uid);
                }
            }
            Event::PrimaryJustReleased => {
                if let Some(target) = self.profiles_target.get(&profile_uid) {
                    let entry = self.widgets.get_mut(target).unwrap();
                    entry.widget.handle_event(ctx, &Event::PrimaryJustReleased);
                }
            }
            Event::Cancel => {
                if let Some(focus) = self.profiles_focus.get(&profile_uid) {
                    let entry = self.widgets.get_mut(focus).unwrap();
                    entry.widget.handle_event(ctx, &Event::LooseFocus);
                    self.profiles_focus.remove(&profile_uid);
                }
            }
            Event::CursorMoved { position } => {
                // Update target
                let previous = self.profiles_target.get(&profile_uid).copied();
                let mut target = None;
                for (uid, entry) in self.widgets.iter() {
                    if entry.widget.extent().contains(*position) {
                        target = Some(*uid);
                        break;
                    }
                }
                if let Some(target) = target {
                    self.profiles_target.insert(profile_uid, target);
                } else {
                    self.profiles_target.remove(&profile_uid);
                }
                // Generate enter/leave events
                if previous != target {
                    if let Some(previous) = previous {
                        let entry = self.widgets.get_mut(&previous).unwrap();
                        entry.widget.handle_event(ctx, &Event::Leave);
                    }
                    if let Some(target) = target {
                        let entry = self.widgets.get_mut(&target).unwrap();
                        entry.widget.handle_event(ctx, &Event::Enter);
                    }
                }
                // Forward event
                if let Some(focus) = self.profiles_focus.get(&profile_uid) {
                    // Simply forward the event
                    let entry = self.widgets.get_mut(focus).unwrap();
                    entry.widget.handle_event(ctx, event);
                }
            }
            Event::SelectionMoved { direction } => {
                // Check focus
                if let Some(focus) = self.profiles_focus.get(&profile_uid) {
                    // Simply forward the event
                    let entry = self.widgets.get_mut(focus).unwrap();
                    entry.widget.handle_event(ctx, event);
                } else {
                    // Change target
                    if let Some(target) = self.profiles_target.get(&profile_uid).copied() {
                        if let Some(next) = self
                            .widgets
                            .get_mut(&target)
                            .unwrap()
                            .navigation
                            .get(*direction)
                        {
                            self.profiles_target.insert(profile_uid, next);
                            self.widgets
                                .get_mut(&target)
                                .unwrap()
                                .widget
                                .handle_event(ctx, &Event::Leave);
                            let next_entry = self.widgets.get_mut(&next).unwrap();
                            ctx.user
                                .lerp_selection_extent(next_entry.widget.extent(), ctx.time);
                            next_entry.widget.handle_event(ctx, &Event::Enter);
                        }
                    } else if let Some(default) = self.default_target {
                        // Set to default focus
                        self.profiles_target.insert(profile_uid, default);
                        let default_entry = self.widgets.get_mut(&default).unwrap();
                        default_entry.widget.handle_event(ctx, &Event::Enter);
                        ctx.user
                            .lerp_selection_extent(default_entry.widget.extent(), ctx.time);
                    }
                }
            }
            Event::ModeChanged => {
                if let Some(focus) = self.profiles_focus.get(&profile_uid) {
                    // Simply forward the event
                    let entry = self.widgets.get_mut(focus).unwrap();
                    entry.widget.handle_event(ctx, event);
                } else {
                    match ctx.user.mode {
                        InteractionMode::Disabled => {}
                        InteractionMode::Selection => {
                            if let Some(target) = self.profiles_target.get(&profile_uid).copied() {
                                let entry = self.widgets.get_mut(&target).unwrap();
                                ctx.user.set_selection_extent(entry.widget.extent());
                                entry.widget.handle_event(ctx, &Event::Enter);
                            } else if let Some(default) = self.default_target {
                                let entry = self.widgets.get_mut(&default).unwrap();
                                self.profiles_target.insert(profile_uid, default);
                                ctx.user.set_selection_extent(entry.widget.extent());
                                entry.widget.handle_event(ctx, &Event::Enter);
                            } else {
                                ctx.user.set_selection_extent(self.extent());
                            }
                        }
                        InteractionMode::Cursor => {}
                    }
                }
            }
            _ => {}
        }
        true
    }

    fn render(&self, gfx: &mut Graphics, styles: &UIStyleSheet, offset: IVec2, time: f64) {
        // Sort widgets
        let mut entries = self.widgets.values().collect::<Vec<_>>();
        entries.sort_by(|a, b| a.z_index.cmp(&b.z_index));

        // Render widgets
        for entry in entries {
            entry.widget.render(gfx, styles, offset, time);
        }
    }

    fn extent(&self) -> IRect {
        SCREEN_VIEWPORT
    }

    fn is_focusable(&self) -> bool {
        false
    }

    fn is_selectable(&self) -> bool {
        true
    }
}
