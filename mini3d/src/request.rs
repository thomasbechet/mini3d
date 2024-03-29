#[derive(Default)]
pub struct Requests {
    pub(crate) shutdown: bool,
    pub(crate) reload_input_mapping: bool,
}

impl Requests {

    pub fn new() -> Self {
        Self::default()
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