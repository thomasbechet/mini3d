use std::{cell::RefCell, rc::Rc};

use mini3d::storage::provider::StorageProvider;

use crate::virtual_disk::VirtualDisk;

pub(crate) struct WinitStorageProvider(Rc<RefCell<VirtualDisk>>);

impl WinitStorageProvider {
    pub(crate) fn new(disk: Rc<RefCell<VirtualDisk>>) -> Self {
        Self(disk)
    }
}

impl StorageProvider for WinitStorageProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn open(
        &mut self,
        path: &str,
        mode: mini3d::storage::provider::DiskFileMode,
    ) -> Result<
        mini3d::storage::provider::StorageFileHandle,
        mini3d::storage::provider::StorageProviderError,
    > {
        todo!()
    }

    fn close(
        &mut self,
        handle: mini3d::storage::provider::StorageFileHandle,
    ) -> Result<(), mini3d::storage::provider::StorageProviderError> {
        todo!()
    }

    fn encoder(
        &mut self,
        handle: mini3d::storage::provider::StorageFileHandle,
    ) -> Result<&mut dyn mini3d::serialize::Encoder, mini3d::storage::provider::StorageProviderError>
    {
        todo!()
    }

    fn decoder(
        &mut self,
        handle: mini3d::storage::provider::StorageFileHandle,
    ) -> Result<&mut dyn mini3d::serialize::Decoder, mini3d::storage::provider::StorageProviderError>
    {
        todo!()
    }

    fn info(
        &self,
        handle: mini3d::storage::provider::StorageFileHandle,
    ) -> Result<
        mini3d::storage::provider::DiskFileInfo,
        mini3d::storage::provider::StorageProviderError,
    > {
        todo!()
    }

    fn seek(
        &mut self,
        handle: mini3d::storage::provider::StorageFileHandle,
        pos: usize,
    ) -> Result<usize, mini3d::storage::provider::StorageProviderError> {
        todo!()
    }

    fn mount(
        &mut self,
        path: &str,
    ) -> Result<
        mini3d::storage::provider::StorageMountHandle,
        mini3d::storage::provider::StorageProviderError,
    > {
        todo!()
    }

    fn unmount(
        &mut self,
        handle: mini3d::storage::provider::StorageMountHandle,
    ) -> Result<(), mini3d::storage::provider::StorageProviderError> {
        todo!()
    }
}
