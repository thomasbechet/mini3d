use std::collections::HashMap;

use crate::{service::renderer::RendererService, math::rect::IRect};

use super::{event::{InputEvent, ButtonState, CursorEvent, TextEvent}, cursor::Cursor, binding::{Button, Axis}, control_layout::{ControlLayout, ControlId}, direction::Direction, input::{InputName, ButtonInput, RangeInput, RangeType}};

pub enum ControlMode {
    Selection { selected_area: Option<ControlId> },
    Cursor { cursor: Cursor },
}

pub struct InputManager {
    pub buttons: HashMap<InputName, ButtonInput>,
    pub axes: HashMap<InputName, RangeInput>,
    control_layout: Option<ControlLayout>,
    control_mode: ControlMode,
}

impl InputManager {

    /// Reset button states and mouse motion
    pub(crate) fn prepare_dispatch(&mut self) {

        // Save the previous button state
        for (_, button) in self.buttons.iter_mut() {
            button.was_pressed = button.pressed;
        }

        // Reset the mouse motion for the current frame
        if let ControlMode::Cursor { cursor } = &mut self.control_mode {
            cursor.reset_motion();
        }
    }

    /// Process input events
    pub(crate) fn dispatch_event(&mut self, event: &InputEvent) {
        match event {
            InputEvent::Button(button_event) => {
                if let Some(action) = self.buttons.get_mut(button_event.name) {
                    match button_event.state {
                        ButtonState::Pressed => {
                            action.pressed = true;
                        },
                        ButtonState::Released => {
                            action.pressed = false;
                        },
                    }
                }
            },
            InputEvent::Axis(axis_event) => {
                if let Some(axis) = self.axes.get_mut(axis_event.name) {
                    axis.set_value(axis_event.value);
                }
            },
            InputEvent::Text(text_event) => {
                match text_event {
                    TextEvent::Character(_char) => {
                        
                    },
                    TextEvent::String(_string) => {
                        
                    },
                }
            },
            InputEvent::Cursor(cursor_event) => {
                if let ControlMode::Cursor { cursor } = &mut self.control_mode {
                    match cursor_event {
                        CursorEvent::Move { delta } => {
                            cursor.translate(*delta);
                        },
                        CursorEvent::Update { position } => {
                            cursor.set_position(*position);
                        },
                    }
                }   
            },
        }
    }

    /// Update selections
    pub(crate) fn update(&mut self) {

        // Check selection mode
        if self.buttons.get(Button::SWITCH_SELECTION_MODE).unwrap().is_just_pressed() {
            match self.control_mode {
                ControlMode::Selection { selected_area: _ } => {
                    self.control_mode = ControlMode::Cursor { cursor: Cursor::new() };
                },
                ControlMode::Cursor { cursor: _ } => {
                    self.control_mode = ControlMode::Selection { selected_area: None }
                },
            }
        }

        // Selection behaviour differ with the cursor mode
        match &mut self.control_mode {
            ControlMode::Selection { selected_area } => {

                // Handle movement
                if let Some(input_layout) = &mut self.control_layout {
                    for direction in Direction::iterator() {
                        if self.buttons.get(Button::from_direction(direction)).unwrap().is_just_pressed() {
                            let id = input_layout.get_control_from_direction(*selected_area, direction);
                            // Update the selected area
                            *selected_area = id;
                        }
                    }
                }

                // Handle click
                if self.buttons.get(Button::CLICK).unwrap().is_just_pressed() {
                    if let Some(id) = selected_area {
                        println!("{}", id);
                    }
                }
            },
            ControlMode::Cursor { cursor } => {

                // Handle click
                if self.buttons.get(Button::CLICK).unwrap().is_just_pressed() {
                    if let Some(input_layout) = &self.control_layout {
                        let id = input_layout.get_control_from_position(cursor.screen_position());
                        if let Some(id) = id {
                            println!("{}", id);
                        }
                    }
                }
            },
        }
    }

    pub(crate) fn render(&self, renderer: &mut impl RendererService) {
        match &self.control_mode {
            ControlMode::Selection { selected_area } => {
                if self.control_layout.is_some() && selected_area.is_some() {
                    let input_layout = self.control_layout.as_ref().unwrap();
                    let extent = input_layout.get_control_extent(selected_area.unwrap());
                    if let Some(extent) = extent {
                        renderer.draw_rect(extent);
                    }
                }
            },
            ControlMode::Cursor { cursor } => {
                let sp = cursor.screen_position();
                renderer.fill_rect(IRect::new(sp.x, sp.y, 2, 2));
            },
        }
    }
}

impl Default for InputManager {
    fn default() -> Self {
        InputManager {
            buttons: HashMap::from([
                (Button::UP, ButtonInput::new()),
                (Button::DOWN, ButtonInput::new()),
                (Button::LEFT, ButtonInput::new()),
                (Button::RIGHT, ButtonInput::new()),
                (Button::CLICK, ButtonInput::new()),
                (Button::SWITCH_SELECTION_MODE, ButtonInput::new()),
            ]),
            axes: HashMap::from([
                (Axis::CURSOR_X, RangeInput::new(RangeType::Infinite)),
                (Axis::CURSOR_Y, RangeInput::new(RangeType::Infinite)),
            ]),
            control_layout: Some(ControlLayout::new()),
            control_mode: ControlMode::Selection { selected_area: None },
        }
    }
}