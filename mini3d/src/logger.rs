use self::level::LogLevel;

pub mod level;
pub mod server;

#[derive(Default)]
pub struct LoggerManager {
    pub(crate) max_level: LogLevel,
}
