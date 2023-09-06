use crate::system::{event::ImportAssetEvent, server::SystemServer, SystemManager};

pub struct ExclusiveSystemAPI<'a> {
    pub(crate) manager: &'a mut SystemManager,
    pub(crate) server: &'a mut dyn SystemServer,
}

impl<'a> ExclusiveSystemAPI<'a> {
    pub fn poll_import(&mut self) -> Option<ImportAssetEvent> {
        self.server.poll_imports()
    }
}

pub struct ParallelSystemAPI<'a> {
    pub(crate) manager: &'a SystemManager,
    pub(crate) server: &'a dyn SystemServer,
}
