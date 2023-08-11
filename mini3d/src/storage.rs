use self::backend::StorageBackend;

pub mod backend;

pub struct StorageManager {
    backend: Box<dyn StorageBackend>,
}

impl StorageManager {
    pub(crate) fn new(backend: impl StorageBackend + 'static) -> Self {
        Self {
            backend: Box::new(backend),
        }
    }
}
