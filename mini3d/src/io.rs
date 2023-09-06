use crate::{
    network::server::NetworkServer, storage::server::StorageServer, utils::slotmap::DenseSlotMap,
};

pub struct IOFile;

pub(crate) enum IOFileKind {
    File,
    Directory,
    Socket,
}

struct IOFileEntry {}

pub(crate) struct IOManager {
    storage: Box<dyn StorageServer>,
    network: Box<dyn NetworkServer>,
    files: DenseSlotMap<IOFileEntry>,
}
