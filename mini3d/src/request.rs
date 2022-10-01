pub struct AppRequests {
    pub(crate) shutdown: bool,
    pub(crate) reload_bindings: bool,
}

impl AppRequests {

    pub fn new() -> Self {
        Self { shutdown: false, reload_bindings: false }
    }

    pub fn shutdown(&self) -> bool {
        self.shutdown
    }

    pub fn reload_bindings(&self) -> bool {
        self.reload_bindings
    }

    pub fn reset(&mut self) {
        self.shutdown = false;
        self.reload_bindings = false;
    }
}