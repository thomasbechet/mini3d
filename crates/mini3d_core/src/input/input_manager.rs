use std::collections::HashMap;

use crate::{service::renderer::RendererService, math::rect::IRect};

use super::{event::{InputEvent, ButtonState, CursorEvent, TextEvent}, cursor::Cursor, binding::{Button, Axis}, input_layout::{InputLayout, AreaId, self}, direction::Direction};

/// Store input state
pub struct ButtonInput {
    /// The button is pressed or released
    pub pressed: bool,
    /// Keep the previous state to detect just pressed and released
    pub(crate) was_pressed: bool,
}

impl ButtonInput {

    pub fn new() -> Self {
        ButtonInput { pressed: false, was_pressed: false }
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    pub fn is_released(&self) -> bool {
        !self.pressed
    }

    pub fn is_just_pressed(&self) -> bool {
        self.pressed && !self.was_pressed
    }

    pub fn is_just_released(&self) -> bool {
        !self.pressed && self.was_pressed
    }
}

pub enum RangeType {
    Clamped { min: f32, max: f32 },
    Normalized { norm: f32 },
    ClampedNormalized { min: f32, max: f32, norm: f32 },
    Infinite,
}

pub struct RangeInput {
    pub value: f32,
    pub range: RangeType,
}

impl RangeInput {
    pub fn new(range: RangeType) -> Self {
        RangeInput { value: 0.0, range: range }
    }

    pub fn set_value(&mut self, value: f32) {
        self.value = match self.range {
            RangeType::Clamped { min, max } => {
                value.max(min).min(max)
            },
            RangeType::Normalized { norm } => {
                value / norm
            },
            RangeType::ClampedNormalized { min, max, norm } => {
                value.max(min).min(max) / norm
            },
            RangeType::Infinite => {
                value
            },
        }
    }
}

pub type InputName = &'static str;

pub enum SelectionMode {
    Area { selected_area: Option<AreaId> },
    Cursor { cursor: Cursor },
}

pub struct InputManager {
    pub buttons: HashMap<InputName, ButtonInput>,
    pub axes: HashMap<InputName, RangeInput>,
    input_layout: Option<InputLayout>,
    selection_mode: SelectionMode,
}

impl InputManager {

    /// Reset button states and mouse motion
    pub(crate) fn prepare_dispatch(&mut self) {

        // Save the previous button state
        for (_, button) in self.buttons.iter_mut() {
            button.was_pressed = button.pressed;
        }

        // Reset the mouse motion for the current frame
        if let SelectionMode::Cursor { cursor } = &mut self.selection_mode {
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
                if let SelectionMode::Cursor { cursor } = &mut self.selection_mode {
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
            match self.selection_mode {
                SelectionMode::Area { selected_area: _ } => {
                    self.selection_mode = SelectionMode::Cursor { cursor: Cursor::new() };
                },
                SelectionMode::Cursor { cursor: _ } => {
                    self.selection_mode = SelectionMode::Area { selected_area: None }
                },
            }
        }

        // Selection behaviour differ with the cursor mode
        match &mut self.selection_mode {
            SelectionMode::Area { selected_area } => {

                // Handle movement
                if let Some(input_layout) = &mut self.input_layout {
                    for direction in Direction::iterator() {
                        if self.buttons.get(Button::from_direction(direction)).unwrap().is_just_pressed() {
                            let id = input_layout.move_next(*selected_area, direction);
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
            SelectionMode::Cursor { cursor } => {

                // Handle click
                if self.buttons.get(Button::CLICK).unwrap().is_just_pressed() {
                    if let Some(input_layout) = &self.input_layout {
                        let id = input_layout.find_area(cursor.screen_position());
                        if let Some(id) = id {
                            println!("{}", id);
                        }
                    }
                }
            },
        }
    }

    pub(crate) fn render(&self, renderer: &mut impl RendererService) {
        match &self.selection_mode {
            SelectionMode::Area { selected_area } => {
                if self.input_layout.is_some() && selected_area.is_some() {
                    let input_layout = self.input_layout.as_ref().unwrap();
                    let extent = input_layout.get_area_extent(selected_area.unwrap());
                    if let Some(extent) = extent {
                        renderer.draw_rect(extent);
                    }
                }
            },
            SelectionMode::Cursor { cursor } => {
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
            input_layout: Some(InputLayout::new()),
            selection_mode: SelectionMode::Area { selected_area: None },
        }
    }
}