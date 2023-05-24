use glam::{IVec2, Vec2};
use mini3d_derive::Serialize;

use crate::{renderer::{graphics::Graphics, color::Color, SCREEN_VIEWPORT, SCREEN_CENTER}, math::rect::IRect, uid::UID};

use super::event::{Event, Direction};

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum InteractionMode {
    Disabled,
    Selection,
    Cursor,
}

fn alpha(last_time: f64, time: f64) -> f64 {
    let mut x = (time - last_time) / 0.3;
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

#[derive(Serialize)]
pub struct UIUser {
    
    pub(crate) name: String,
    pub(crate) mode: InteractionMode,
    pub(crate) events: Vec<Event>,
    pub(crate) locked: bool,
    pub(crate) extent: IRect,
    
    selection_extent: IRect,
    selection_source_extent: IRect,
    selection_source_time: f64,
    cursor_position: Vec2,
    cursor_previous_position: Vec2,
    primary_pressed: bool,
}

impl UIUser {

    pub(crate) fn new(name: &str, extent: IRect) -> Self {
        Self {
            name: name.to_string(),
            mode: InteractionMode::Disabled,
            events: Vec::new(),
            locked: false,
            extent,

            selection_extent: SCREEN_VIEWPORT,
            selection_source_extent: SCREEN_VIEWPORT,
            selection_source_time: 0.0,
            cursor_position: SCREEN_CENTER.as_vec2(),
            cursor_previous_position: SCREEN_CENTER.as_vec2(),
            primary_pressed: false,
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

    pub(crate) fn lerp_selection_extent(&mut self, target_extent: IRect, time: f64) {
        self.selection_source_extent = self.selection_extent;
        self.selection_source_time = time;
        self.selection_extent = target_extent;
    }

    pub(crate) fn render(&self, gfx: &mut Graphics, time: f64) {
        // Display selection box or cursor
        // TODO: each profile have an associated color
        // TODO: two selection box on the same extent have special design
        if !self.locked {
            match &self.mode {
                InteractionMode::Disabled => {},
                InteractionMode::Selection => {
                    let lerp_extent = self.selection_source_extent.lerp(self.selection_extent, alpha(self.selection_source_time, time) as f32);
                    render_selection(lerp_extent, gfx, time);
                },
                InteractionMode::Cursor => {
                    render_cursor(self.cursor_position.as_ivec2(), gfx, time);
                },
            }
        }
    }

    pub fn warp_cursor(&mut self, position: Vec2) {
        self.cursor_position = position.clamp(self.extent.tl().as_vec2(), self.extent.br().as_vec2());
        if self.cursor_position != self.cursor_previous_position {
            self.cursor_previous_position = self.cursor_position;
            self.events.push(Event::CursorMoved { position: self.cursor_position.as_ivec2() });
        }
    }

    pub fn move_cursor(&mut self, delta: Vec2) {
        self.cursor_position += delta;
        self.cursor_position = self.cursor_position.clamp(self.extent.tl().as_vec2(), self.extent.br().as_vec2());
        if self.cursor_position != self.cursor_previous_position {
            self.cursor_previous_position = self.cursor_position;
            // self.cursor_position = position.as_vec2();
            self.events.push(Event::CursorMoved { position: self.cursor_position.as_ivec2() });
        }
    }

    pub fn move_selection(&mut self, direction: Direction) {
        self.events.push(Event::SelectionMoved { direction });
    }

    pub fn scroll(&mut self, value: f32) {
        self.events.push(Event::Scroll { value });
    }

    pub fn press_primary(&mut self) {
        if !self.primary_pressed {
            self.primary_pressed = true;
            self.events.push(Event::PrimaryJustPressed);
        }
    }

    pub fn release_primary(&mut self) {
        if self.primary_pressed {
            self.primary_pressed = false;
            self.events.push(Event::PrimaryJustReleased);
        }
    }

    pub fn cancel(&mut self) {
        self.events.push(Event::Cancel);
    }
}