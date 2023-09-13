use super::level::LogLevel;
use core::fmt::Arguments;

#[allow(unused_variables)]
pub trait LoggerServer {
    fn log(&mut self, args: Arguments<'_>, level: LogLevel, source: Option<(&'static str, u32)>) {}
}

#[derive(Default)]
pub struct DummyLoggerServer;

impl LoggerServer for DummyLoggerServer {}
