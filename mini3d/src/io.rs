use crate::{
    disk::provider::DiskProvider, network::provider::NetworkProvider, utils::slotmap::DenseSlotMap,
};

pub struct IOFile;

pub(crate) enum IOFileKind {
    File,
    Directory,
    Socket,
}

struct IOFileEntry {}

pub struct IOManager {
    disk: Box<dyn DiskProvider>,
    network: Box<dyn NetworkProvider>,
    files: DenseSlotMap<IOFileEntry>,
}

impl IOManager {
    // pub
}
