use std::{cell::RefCell, fmt::Arguments};

use self::{level::LogLevel, provider::LoggerProvider};

pub mod level;
pub mod provider;

#[derive(Default)]
pub struct LoggerManager {
    provider: RefCell<Box<dyn LoggerProvider>>,
    max_level: LogLevel,
}

impl LoggerManager {
    pub(crate) fn set_provider(&mut self, server: Box<dyn LoggerProvider>) {
        self.provider.get_mut().on_disconnect();
        self.provider = RefCell::new(server);
        self.provider.get_mut().on_connect();
    }

    pub fn log(&self, args: Arguments<'_>, level: LogLevel, source: Option<(&'static str, u32)>) {
        if level <= self.max_level {
            // TODO: handle proper concurrent logging
            self.provider.borrow_mut().log(args, level, source);
        }
    }

    pub fn set_max_level(&mut self, level: LogLevel) {
        self.max_level = level;
    }
}
