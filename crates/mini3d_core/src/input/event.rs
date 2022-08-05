#[derive(Copy, Clone)]
pub enum ActionState {
    Pressed,
    Released,
}

pub struct ActionEvent {
    pub name: &'static str,
    pub state: ActionState,
}

pub struct AxisEvent {
    pub name: &'static str,
    pub value: f32,
}

pub enum TextEvent {
    Character(char),
    String(String),
}

pub enum InputEvent {
    Action(ActionEvent),
    Axis(AxisEvent),
    Text(TextEvent),
}