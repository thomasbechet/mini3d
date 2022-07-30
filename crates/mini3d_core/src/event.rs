use crate::input::event::InputEvent;

pub enum Event {
    CloseRequested,
    Input(InputEvent),
}