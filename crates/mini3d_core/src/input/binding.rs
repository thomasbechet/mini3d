use super::direction::Direction;

pub struct Button;

impl Button {
    pub const UP: &'static str = "up";
    pub const DOWN: &'static str = "down";
    pub const LEFT: &'static str = "left";
    pub const RIGHT: &'static str = "right";

    pub fn from_direction(direction: Direction) -> &'static str {
        match direction {
            Direction::Up => Button::UP,
            Direction::Down => Button::DOWN,
            Direction::Left => Button::LEFT,
            Direction::Right => Button::RIGHT,
        }
    }

    pub const CLICK: &'static str = "click";
    pub const SWITCH_SELECTION_MODE: &'static str = "switch_selection_mode";
}

pub struct Axis;

impl Axis {
    pub const CURSOR_X: &'static str = "cursor_x";
    pub const CURSOR_Y: &'static str = "cursor_y";
}