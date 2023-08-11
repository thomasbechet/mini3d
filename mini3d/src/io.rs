use crate::{
    network::backend::NetworkBackend, storage::backend::StorageBackend,
    utils::slotmap::DenseSlotMap,
};

pub struct IOFile;

pub(crate) enum IOFileKind {
    File,
    Directory,
    Socket,
}

struct IOFileEntry {}

pub(crate) struct IOManager {
    storage: Box<dyn StorageBackend>,
    network: Box<dyn NetworkBackend>,
    files: DenseSlotMap<IOFileEntry>,
}
