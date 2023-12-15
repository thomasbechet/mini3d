use alloc::boxed::Box;
use mini3d_derive::Error;

use crate::{
    define_provider_handle,
    serialize::{Decoder, Encoder},
};

#[derive(Debug, Error)]
pub enum StorageProviderError {
    #[error("Unknown error")]
    Unknown,
}

define_provider_handle!(DiskFileHandle);
define_provider_handle!(DiskMountHandle);

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
pub trait DiskProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);
    fn open(
        &mut self,
        path: &str,
        mode: DiskFileMode,
    ) -> Result<DiskFileHandle, StorageProviderError>;
    fn close(&mut self, handle: DiskFileHandle) -> Result<(), StorageProviderError>;
    fn encoder(&mut self, handle: DiskFileHandle)
        -> Result<&mut dyn Encoder, StorageProviderError>;
    fn decoder(&mut self, handle: DiskFileHandle)
        -> Result<&mut dyn Decoder, StorageProviderError>;
    fn info(&self, handle: DiskFileHandle) -> Result<DiskFileInfo, StorageProviderError>;
    fn seek(&mut self, handle: DiskFileHandle, pos: usize) -> Result<usize, StorageProviderError>;
    fn mount(&mut self, path: &str) -> Result<DiskMountHandle, StorageProviderError>;
    fn unmount(&mut self, handle: DiskMountHandle) -> Result<(), StorageProviderError>;
}

#[derive(Default)]
pub struct PassiveStorageProvider;

impl DiskProvider for PassiveStorageProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}
    fn open(
        &mut self,
        path: &str,
        mode: DiskFileMode,
    ) -> Result<DiskFileHandle, StorageProviderError> {
        Ok(Default::default())
    }
    fn close(&mut self, handle: DiskFileHandle) -> Result<(), StorageProviderError> {
        Ok(())
    }
    fn encoder(
        &mut self,
        handle: DiskFileHandle,
    ) -> Result<&mut dyn Encoder, StorageProviderError> {
        Err(StorageProviderError::Unknown)
    }
    fn decoder(
        &mut self,
        handle: DiskFileHandle,
    ) -> Result<&mut dyn Decoder, StorageProviderError> {
        Err(StorageProviderError::Unknown)
    }
    fn info(&self, handle: DiskFileHandle) -> Result<DiskFileInfo, StorageProviderError> {
        Ok(DiskFileInfo {
            name: "",
            kind: DiskFileKind::File,
        })
    }
    fn seek(&mut self, handle: DiskFileHandle, pos: usize) -> Result<usize, StorageProviderError> {
        Ok(0)
    }
    fn mount(&mut self, path: &str) -> Result<DiskMountHandle, StorageProviderError> {
        Ok(Default::default())
    }
    fn unmount(&mut self, handle: DiskMountHandle) -> Result<(), StorageProviderError> {
        Ok(())
    }
}

impl Default for Box<dyn DiskProvider> {
    fn default() -> Self {
        Box::<PassiveStorageProvider>::default()
    }
}
