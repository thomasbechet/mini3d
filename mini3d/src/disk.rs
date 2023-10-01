use self::provider::DiskProvider;

pub mod provider;

#[derive(Default)]
pub struct StorageManager {
    provider: Box<dyn DiskProvider>,
}

impl StorageManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn DiskProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }
}
