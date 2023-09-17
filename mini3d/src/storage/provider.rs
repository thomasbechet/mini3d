use mini3d_derive::Error;

use crate::{
    define_server_handle,
    serialize::{Decoder, Encoder},
};

#[derive(Debug, Error)]
pub enum StorageProviderError {
    #[error("Unknown error")]
    Unknown,
}

define_server_handle!(StorageFileHandle);
define_server_handle!(StorageMountHandle);

pub enum DiskFileKind {
    File,
    Directory,
}

pub enum DiskFileMode {
    Read,
    Create,
    Append,
}

pub struct DiskFileInfo<'a> {
    name: &'a str,
    kind: DiskFileKind,
}

#[allow(unused_variables)]
pub trait StorageProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);
    fn open(
        &mut self,
        path: &str,
        mode: DiskFileMode,
    ) -> Result<StorageFileHandle, StorageProviderError>;
    fn close(&mut self, handle: StorageFileHandle) -> Result<(), StorageProviderError>;
    fn encoder(
        &mut self,
        handle: StorageFileHandle,
    ) -> Result<&mut dyn Encoder, StorageProviderError>;
    fn decoder(
        &mut self,
        handle: StorageFileHandle,
    ) -> Result<&mut dyn Decoder, StorageProviderError>;
    fn info(&self, handle: StorageFileHandle) -> Result<DiskFileInfo, StorageProviderError>;
    fn seek(
        &mut self,
        handle: StorageFileHandle,
        pos: usize,
    ) -> Result<usize, StorageProviderError>;
    fn mount(&mut self, path: &str) -> Result<StorageMountHandle, StorageProviderError>;
    fn unmount(&mut self, handle: StorageMountHandle) -> Result<(), StorageProviderError>;
}

#[derive(Default)]
pub struct PassiveStorageProvider;

impl StorageProvider for PassiveStorageProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}
    fn open(
        &mut self,
        path: &str,
        mode: DiskFileMode,
    ) -> Result<StorageFileHandle, StorageProviderError> {
        Ok(Default::default())
    }
    fn close(&mut self, handle: StorageFileHandle) -> Result<(), StorageProviderError> {
        Ok(())
    }
    fn encoder(
        &mut self,
        handle: StorageFileHandle,
    ) -> Result<&mut dyn Encoder, StorageProviderError> {
        Err(StorageProviderError::Unknown)
    }
    fn decoder(
        &mut self,
        handle: StorageFileHandle,
    ) -> Result<&mut dyn Decoder, StorageProviderError> {
        Err(StorageProviderError::Unknown)
    }
    fn info(&self, handle: StorageFileHandle) -> Result<DiskFileInfo, StorageProviderError> {
        Ok(DiskFileInfo {
            name: "",
            kind: DiskFileKind::File,
        })
    }
    fn seek(
        &mut self,
        handle: StorageFileHandle,
        pos: usize,
    ) -> Result<usize, StorageProviderError> {
        Ok(0)
    }
    fn mount(&mut self, path: &str) -> Result<StorageMountHandle, StorageProviderError> {
        Ok(Default::default())
    }
    fn unmount(&mut self, handle: StorageMountHandle) -> Result<(), StorageProviderError> {
        Ok(())
    }
}

impl Default for Box<dyn StorageProvider> {
    fn default() -> Self {
        Box::<PassiveStorageProvider>::default()
    }
}
