use std::{cell::RefCell, rc::Rc};

use mini3d::disk::provider::DiskProvider;

use crate::virtual_disk::VirtualDisk;

pub(crate) struct WinitStorageProvider(Rc<RefCell<VirtualDisk>>);

impl WinitStorageProvider {
    pub(crate) fn new(disk: Rc<RefCell<VirtualDisk>>) -> Self {
        Self(disk)
    }
}

impl DiskProvider for WinitStorageProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn open(
        &mut self,
        path: &str,
        mode: mini3d::disk::provider::DiskFileMode,
    ) -> Result<mini3d::disk::provider::DiskFileHandle, mini3d::disk::provider::StorageProviderError>
    {
        todo!()
    }

    fn close(
        &mut self,
        handle: mini3d::disk::provider::DiskFileHandle,
    ) -> Result<(), mini3d::disk::provider::StorageProviderError> {
        todo!()
    }

    fn encoder(
        &mut self,
        handle: mini3d::disk::provider::DiskFileHandle,
    ) -> Result<&mut dyn mini3d::serialize::Encoder, mini3d::disk::provider::StorageProviderError>
    {
        todo!()
    }

    fn decoder(
        &mut self,
        handle: mini3d::disk::provider::DiskFileHandle,
    ) -> Result<&mut dyn mini3d::serialize::Decoder, mini3d::disk::provider::StorageProviderError>
    {
        todo!()
    }

    fn info(
        &self,
        handle: mini3d::disk::provider::DiskFileHandle,
    ) -> Result<mini3d::disk::provider::DiskFileInfo, mini3d::disk::provider::StorageProviderError>
    {
        todo!()
    }

    fn seek(
        &mut self,
        handle: mini3d::disk::provider::DiskFileHandle,
        pos: usize,
    ) -> Result<usize, mini3d::disk::provider::StorageProviderError> {
        todo!()
    }

    fn mount(
        &mut self,
        path: &str,
    ) -> Result<mini3d::disk::provider::DiskMountHandle, mini3d::disk::provider::StorageProviderError>
    {
        todo!()
    }

    fn unmount(
        &mut self,
        handle: mini3d::disk::provider::DiskMountHandle,
    ) -> Result<(), mini3d::disk::provider::StorageProviderError> {
        todo!()
    }
}
