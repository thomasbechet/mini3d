use anyhow::Result;
use glam::{Vec2, IVec2};
use serde::{Serialize, Deserialize};

use crate::{uid::UID, math::rect::IRect, renderer::{graphics::Graphics, SCREEN_VIEWPORT, color::Color, SCREEN_CENTER}, context::input::InputContext};

use super::event::{Direction, Event};

#[derive(Serialize, Deserialize)]
pub struct ProfileInputs {
    
    // Buttons
    pub click: UID,
    pub back: UID,
    pub up: UID,
    pub down: UID,
    pub left: UID,
    pub right: UID,
    
    // Axis
    pub scroll: UID,
    pub cursor_x: UID,
    pub cursor_y: UID,
    pub cursor_motion_x: UID,
    pub cursor_motion_y: UID,
}

// #[derive(Serialize, Deserialize)]
// pub(crate) struct VisualSelection {
//     source_extent: IRect,
//     target_extent: IRect,
//     source_time: f64,
// }

// impl VisualSelection {
//     pub(crate) fn new(extent: IRect) -> Self {
//         Self { source_extent: extent, target_extent: extent, source_time: 0.0 }
//     }
// }

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) enum ProfileMode {
    Disabled,
    Selection,
    Cursor,
}

fn alpha(last_time: f64, time: f64) -> f64 {
    let mut x = (time - last_time) / 0.4;
    x = 1.0 - (1.0 - x).powi(3);
    x.clamp(0.0, 1.0)
}

fn render_selection(extent: IRect, gfx: &mut Graphics, time: f64) {
    let offset = i32::from((time % 1.0) > 0.5);
    let length = 2;

    let tl = extent.tl() + IVec2::new(-offset, -offset);
    let tr = extent.tr() + IVec2::new(offset, -offset); 
    let bl = extent.bl() + IVec2::new(-offset, offset); 
    let br = extent.br() + IVec2::new(offset, offset); 

    gfx.draw_hline(tl.y, tl.x, tl.x + length, Color::WHITE);
    gfx.draw_vline(tl.x, tl.y, tl.y + length, Color::WHITE);
    
    gfx.draw_hline(tr.y, tr.x - length, tr.x, Color::WHITE);
    gfx.draw_vline(tr.x, tr.y, tr.y + length, Color::WHITE);

    gfx.draw_hline(bl.y, bl.x, bl.x + length, Color::WHITE);
    gfx.draw_vline(bl.x, bl.y - length, bl.y, Color::WHITE);

    gfx.draw_hline(br.y, br.x - length, br.x, Color::WHITE);
    gfx.draw_vline(br.x, br.y - length, br.y, Color::WHITE);
}

fn render_cursor(position: IVec2, gfx: &mut Graphics, _time: f64) {
    gfx.draw_hline(position.y, position.x - 1, position.x + 1, Color::WHITE);
    gfx.draw_vline(position.x, position.y - 1, position.y + 1, Color::WHITE);
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Profile {
    pub(crate) name: String,
    pub(crate) mode: ProfileMode,
    pub(crate) inputs: ProfileInputs,
    
    selection_extent: IRect,
    selection_source_extent: IRect,
    selection_source_time: f64,
    cursor_position: Vec2,
    cursor_previous_axis_position: Vec2,
}

impl Profile {

    pub(crate) fn new(name: &str, inputs: ProfileInputs) -> Self {
        Self {
            name: name.to_string(),
            mode: ProfileMode::Disabled,
            inputs,
            selection_extent: SCREEN_VIEWPORT,
            selection_source_extent: SCREEN_VIEWPORT,
            selection_source_time: 0.0,
            cursor_position: SCREEN_CENTER.as_vec2(),     
            cursor_previous_axis_position: SCREEN_CENTER.as_vec2(),     
        }
    }

    pub(crate) fn uid(&self) -> UID {
        self.name.as_str().into()
    }

    pub(crate) fn set_selection_extent(&mut self, target_extent: IRect) {
        self.selection_source_extent = target_extent;
        self.selection_extent = target_extent;
        self.selection_source_time = 0.0;
    }

    pub(crate) fn move_selection_extent(&mut self, target_extent: IRect, time: f64) {
        self.selection_source_extent = self.selection_extent;
        self.selection_source_time = time;
        self.selection_extent = target_extent;
    }

    pub(crate) fn update(&mut self, input: &InputContext<'_>, events: &mut Vec<Event>) -> Result<()> {
        
        // Cursor inputs
        let cursor_x = input.axis(self.inputs.cursor_x)?.value;
        let cursor_y = input.axis(self.inputs.cursor_y)?.value;
        let cursor_update = cursor_x != self.cursor_previous_axis_position.x || cursor_y != self.cursor_previous_axis_position.y;
        let motion_x = input.axis(self.inputs.cursor_motion_x)?.value;
        let motion_y = input.axis(self.inputs.cursor_motion_y)?.value;
        let motion_update = motion_x != 0.0 || motion_y != 0.0;

        // Update cursor position
        if cursor_update {
            self.cursor_previous_axis_position = Vec2::new(cursor_x, cursor_y);
            let new_position = Vec2::new(cursor_x, cursor_y).clamp(SCREEN_VIEWPORT.tl().as_vec2(), SCREEN_VIEWPORT.br().as_vec2());
            self.cursor_position = new_position;
        } else if motion_update {
            let new_position = (self.cursor_position + Vec2::new(motion_x, motion_y)).clamp(SCREEN_VIEWPORT.tl().as_vec2(), SCREEN_VIEWPORT.br().as_vec2());
            self.cursor_position = new_position;
        }

        // Direction inputs
        let direction = if input.action(self.inputs.up)?.is_just_pressed() {
            Some(Direction::Up)
        } else if input.action(self.inputs.down)?.is_just_pressed() {
            Some(Direction::Down)
        } else if input.action(self.inputs.left)?.is_just_pressed() {
            Some(Direction::Left)
        } else if input.action(self.inputs.right)?.is_just_pressed() {
            Some(Direction::Right)
        } else {
            None
        };

        // Generate mode events
        match self.mode {
            ProfileMode::Disabled => {
                if motion_update || cursor_update {
                    self.mode = ProfileMode::Cursor;
                    events.push(Event::ModeChange);
                    events.push(Event::CursorMove { position: self.cursor_position.as_ivec2() });
                } else if direction.is_some() {
                    self.mode = ProfileMode::Selection;
                    events.push(Event::ModeChange);
                }
            },
            ProfileMode::Selection => {
                if motion_update || cursor_update {
                    self.mode = ProfileMode::Cursor;
                    events.push(Event::ModeChange);
                    if motion_update {
                        self.cursor_position = self.selection_extent.center().as_vec2();
                    }
                    events.push(Event::CursorMove { position: self.cursor_position.as_ivec2() });
                } else if let Some(direction) = direction {
                    events.push(Event::SelectionMove { direction });
                }
            },
            ProfileMode::Cursor => {
                if direction.is_some() {
                    self.mode = ProfileMode::Selection;
                    events.push(Event::ModeChange);
                } else if motion_update || cursor_update {
                    events.push(Event::CursorMove { position: self.cursor_position.as_ivec2() });
                }
            },
        }
    
        // Primary input
        let action = input.action(self.inputs.click)?;
        if action.is_just_pressed() {
            events.push(Event::PrimaryJustPressed);
        } else if action.is_just_released() {
            events.push(Event::PrimaryJustReleased);
        }

        // Secondary input
        if input.action(self.inputs.back)?.is_just_pressed() {
            events.push(Event::SecondaryJustPressed);
        } else if input.action(self.inputs.back)?.is_just_released() {
            events.push(Event::SecondaryJustReleased);
        }

        Ok(())
    }

    pub fn render(&self, gfx: &mut Graphics, time: f64) {
        // Display selection box or cursor
        // TODO: each profile have an associated color
        // TODO: two selection box on the same extent have special design
        match &self.mode {
            ProfileMode::Disabled => {},
            ProfileMode::Selection => {
                let lerp_extent = self.selection_source_extent.lerp(&self.selection_extent, alpha(self.selection_source_time, time) as f32);
                render_selection(lerp_extent, gfx, time);
            },
            ProfileMode::Cursor => {
                render_cursor(self.cursor_position.as_ivec2(), gfx, time);
            },
        }
    }
}