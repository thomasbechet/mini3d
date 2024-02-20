use alloc::boxed::Box;
use mini3d_utils::{slot_map_key, slotmap::DenseSlotMap};

use crate::provider::DiskProvider;

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
    files: DenseSlotMap<IOFileHandle, IOFileEntry>,
}

impl IOManager {
    // pub
}
