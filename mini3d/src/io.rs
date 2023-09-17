use crate::{
    network::provider::NetworkProvider, storage::provider::StorageProvider,
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
    storage: Box<dyn StorageProvider>,
    network: Box<dyn NetworkProvider>,
    files: DenseSlotMap<IOFileEntry>,
}
