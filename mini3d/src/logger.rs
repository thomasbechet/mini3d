use core::{cell::RefCell, fmt::Arguments};

use alloc::boxed::Box;

use self::{level::LogLevel, provider::LoggerProvider};

pub mod level;
pub mod provider;

#[derive(Default)]
pub struct LoggerManager {
    provider: RefCell<Box<dyn LoggerProvider>>,
    max_level: LogLevel,
}

impl LoggerManager {
    pub(crate) fn set_provider(&mut self, provider: Box<dyn LoggerProvider>) {
        self.provider.get_mut().on_disconnect();
        self.provider = RefCell::new(provider);
        self.provider.get_mut().on_connect();
    }

    pub(crate) fn log(
        &self,
        args: Arguments<'_>,
        level: LogLevel,
        source: Option<(&'static str, u32)>,
    ) {
        if level <= self.max_level {
            // TODO: handle proper concurrent logging
            self.provider.borrow_mut().log(args, level, source);
        }
    }

    pub(crate) fn set_max_level(&mut self, level: LogLevel) {
        self.max_level = level;
    }
}
