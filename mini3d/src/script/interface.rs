use crate::utils::uid::UID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InterfaceId(u32);

pub(crate) struct Interface {
    pub(crate) asset: UID,
}

#[derive(Default)]
pub(crate) struct InterfaceTable {
    interfaces: Vec<Interface>,
}

impl InterfaceTable {}
