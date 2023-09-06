use mini3d_derive::Error;

use crate::{
    define_server_handle,
    serialize::{Decoder, Encoder},
};

#[derive(Debug, Error)]
pub enum StorageServerError {
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
pub trait StorageServer {
    fn open(
        &mut self,
        path: &str,
        mode: DiskFileMode,
    ) -> Result<StorageFileHandle, StorageServerError> {
        Ok(Default::default())
    }
    fn close(&mut self, handle: StorageFileHandle) -> Result<(), StorageServerError> {
        Ok(())
    }
    fn encoder(
        &mut self,
        handle: StorageFileHandle,
    ) -> Result<&mut dyn Encoder, StorageServerError> {
        Err(StorageServerError::Unknown)
    }
    fn decoder(
        &mut self,
        handle: StorageFileHandle,
    ) -> Result<&mut dyn Decoder, StorageServerError> {
        Err(StorageServerError::Unknown)
    }
    fn info(&self, handle: StorageFileHandle) -> Result<DiskFileInfo, StorageServerError> {
        Ok(DiskFileInfo {
            name: "",
            kind: DiskFileKind::File,
        })
    }
    fn seek(&mut self, handle: StorageFileHandle, pos: usize) -> Result<usize, StorageServerError> {
        Ok(0)
    }
    fn mount(&mut self, path: &str) -> Result<StorageMountHandle, StorageServerError> {
        Ok(Default::default())
    }
    fn unmount(&mut self, handle: StorageMountHandle) -> Result<(), StorageServerError> {
        Ok(())
    }
}

#[derive(Default)]
pub struct DummyStorageserver;

impl StorageServer for DummyStorageserver {}
