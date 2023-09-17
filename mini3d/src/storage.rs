use self::provider::StorageProvider;

pub mod provider;

#[derive(Default)]
pub struct StorageManager {
    provider: Box<dyn StorageProvider>,
}

impl StorageManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn StorageProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }
}
