use super::level::LogLevel;
use core::fmt::Arguments;

#[allow(unused_variables)]
pub trait LoggerProvider {
    fn on_connect(&mut self);
    fn on_disconnect(&mut self);
    fn log(&mut self, args: Arguments<'_>, level: LogLevel, source: Option<(&'static str, u32)>);
}

#[derive(Default)]
pub struct PassiveLoggerProvider;

impl LoggerProvider for PassiveLoggerProvider {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}
    fn log(
        &mut self,
        _args: Arguments<'_>,
        _level: LogLevel,
        _source: Option<(&'static str, u32)>,
    ) {
    }
}

impl Default for Box<dyn LoggerProvider> {
    fn default() -> Self {
        Box::new(PassiveLoggerProvider)
    }
}
