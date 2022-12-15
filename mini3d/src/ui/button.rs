use serde::{Serialize, Deserialize};

use crate::{uid::UID, renderer::backend::CanvasSpriteHandle};

#[derive(Default, Serialize, Deserialize)]
pub struct Button {
    pressed: bool,
    #[serde(skip)]
    handle: Option<CanvasSpriteHandle>,
}

pub struct ButtonEvent {
    uid: UID,
    pressed: bool,
}

// fn handle_click(uid: UID, widgets: &mut HashMap<UID, Widget>) {
//     let button = widgets.get(&uid).unwrap();
//     button.
// }

impl Button {

}