use crate::{
    disk::provider::DiskProvider, network::provider::NetworkProvider, slot_map_key,
    utils::slotmap::DenseSlotMap,
};

pub struct IOFile;

pub(crate) enum IOFileKind {
    File,
    Directory,
    Socket,
}

struct IOFileEntry {}

slot_map_key!(IOFileHandle);

pub struct IOManager {
    disk: Box<dyn DiskProvider>,
    network: Box<dyn NetworkProvider>,
    files: DenseSlotMap<IOFileHandle, IOFileEntry>,
}

impl IOManager {
    // pub
}
