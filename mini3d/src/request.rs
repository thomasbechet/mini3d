pub struct AppRequests {
    pub(crate) shutdown: bool,
    pub(crate) reload_bindings: bool,
}

impl AppRequests {

    pub fn new() -> Self {
        Self { shutdown: false, reload_bindings: false }
    }

    pub fn shutdown(&mut self) -> bool {
        let yes = self.shutdown;
        self.shutdown = false;
        yes
    }

    pub fn reload_bindings(&mut self) -> bool {
        let yes = self.reload_bindings;
        self.reload_bindings = false;
        yes
    }
}