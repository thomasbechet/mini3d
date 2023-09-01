use crate::system::event::SystemEvent;

pub struct EventAPI<'a> {
    pub system: &'a [SystemEvent],
}
