use glam::IVec2;

use crate::{math::rect::IRect, asset::AssetID};

pub enum ImmediateCommand {
    Print {
        p: IVec2,
        text: String,
        font_id: AssetID,
    },
    DrawLine {
        p0: IVec2,
        p1: IVec2,
    },
    DrawVLine {
        x: i32,
        y0: i32,
        y1: i32,
    },
    DrawHLine {
        y: i32,
        x0: i32,
        x1: i32,
    },
    DrawRect {
        rect: IRect,
    },
    FillRect {
        rect: IRect,
    },
}