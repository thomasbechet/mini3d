use glam::Vec2;
use mini3d_derive::Serialize;

use crate::{
    input::{InputError, InputManager},
    utils::uid::{ToUID, UID},
};

use super::{event::Direction, user::UIUser};

#[derive(Default, Serialize)]
pub struct UIController {
    selection_move: Option<(UID, UID, UID, UID)>,

    cursor_position: Option<(UID, UID)>,
    cursor_motion: Option<(UID, UID)>,

    primary: Option<UID>,
    cancel: Option<UID>,

    previous_cursor_position: Vec2,
}

impl UIController {
    pub fn new() -> Self {
        Self {
            previous_cursor_position: Vec2::ZERO,
            ..Default::default()
        }
    }

    pub fn with_selection_move(
        mut self,
        up: impl ToUID,
        down: impl ToUID,
        left: impl ToUID,
        right: impl ToUID,
    ) -> Self {
        self.selection_move = Some((up.to_uid(), down.to_uid(), left.to_uid(), right.to_uid()));
        self
    }

    pub fn with_cursor_position(mut self, axis_x: impl ToUID, axis_y: impl ToUID) -> Self {
        self.cursor_position = Some((axis_x.to_uid(), axis_y.to_uid()));
        self
    }

    pub fn with_cursor_motion(mut self, axis_x: impl ToUID, axis_y: impl ToUID) -> Self {
        self.cursor_motion = Some((axis_x.to_uid(), axis_y.to_uid()));
        self
    }

    pub fn with_primary(mut self, primary: impl ToUID) -> Self {
        self.primary = Some(primary.to_uid());
        self
    }

    pub fn with_cancel(mut self, cancel: impl ToUID) -> Self {
        self.cancel = Some(cancel.to_uid());
        self
    }

    pub fn update(&mut self, input: &InputManager, user: &mut UIUser) -> Result<(), InputError> {
        if let Some((up, down, left, right)) = self.selection_move {
            if input.action(up)?.is_just_pressed() {
                user.move_selection(Direction::Up);
            } else if input.action(down)?.is_just_pressed() {
                user.move_selection(Direction::Down);
            } else if input.action(left)?.is_just_pressed() {
                user.move_selection(Direction::Left);
            } else if input.action(right)?.is_just_pressed() {
                user.move_selection(Direction::Right);
            }
        }

        if let Some((axis_x, axis_y)) = self.cursor_motion {
            let motion = Vec2::new(input.axis(axis_x)?.value, input.axis(axis_y)?.value);
            if motion.x != 0.0 || motion.y != 0.0 {
                user.move_cursor(motion);
            }
        }

        if let Some((axis_x, axis_y)) = self.cursor_position {
            let position = Vec2::new(input.axis(axis_x)?.value, input.axis(axis_y)?.value);
            if self.previous_cursor_position != position {
                self.previous_cursor_position = position;
                user.warp_cursor(position);
            }
        }

        if let Some(action) = self.primary {
            if input.action(action)?.is_pressed() {
                user.press_primary();
            } else if input.action(action)?.is_released() {
                user.release_primary();
            }
        }

        if let Some(action) = self.cancel {
            if input.action(action)?.is_just_pressed() {
                user.cancel();
            }
        }

        Ok(())
    }
}
