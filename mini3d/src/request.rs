pub struct AppRequests {
    pub(crate) shutdown: bool,
    pub(crate) reload_input_mapping: bool,
}

impl AppRequests {

    pub fn new() -> Self {
        Self { shutdown: false, reload_input_mapping: false }
    }

    pub fn shutdown(&self) -> bool {
        self.shutdown
    }

    pub fn reload_input_mapping(&self) -> bool {
        self.reload_input_mapping
    }

    pub fn reset(&mut self) {
        self.shutdown = false;
        self.reload_input_mapping = false;
    }
}