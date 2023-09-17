use std::fmt::Arguments;

use chrono::{SecondsFormat, Utc};
use colored::Colorize;
use mini3d::logger::{level::LogLevel, provider::LoggerProvider};

pub(crate) struct ConsoleLogger;

impl LoggerProvider for ConsoleLogger {
    fn on_connect(&mut self) {}
    fn on_disconnect(&mut self) {}

    fn log(&mut self, args: Arguments<'_>, level: LogLevel, source: Option<(&'static str, u32)>) {
        let (file, line) = match source {
            Some((file, line)) => (file, line),
            None => ("", 0),
        };
        let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true).cyan();
        let level = match level {
            LogLevel::Info => "INFO".blue().bold(),
            LogLevel::Debug => "DEBUG".yellow().bold(),
            LogLevel::Warning => "WARNING".yellow().bold(),
            LogLevel::Error => "ERROR".red().bold(),
            LogLevel::Critical => "CRITICAL".red().bold(),
        };
        println!(
            "[{}][{}][{}] {}",
            now,
            level,
            format!("{}:{}", file, line).green(),
            args.to_string().bright_black()
        );
    }
}
