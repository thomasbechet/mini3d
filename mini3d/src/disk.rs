use alloc::boxed::Box;

use self::provider::DiskProvider;

pub mod provider;

#[derive(Default)]
pub struct DiskManager {
    provider: Box<dyn DiskProvider>,
}

impl DiskManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn DiskProvider>) {
        self.provider.on_disconnect();
        self.provider = provider;
        self.provider.on_connect();
    }
}
