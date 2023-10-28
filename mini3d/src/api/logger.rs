use std::fmt::Arguments;

use crate::logger::level::LogLevel;

use super::Context;

#[macro_export]
macro_rules! info {
    ($ctx:ident, $($arg:tt)*) => {{
        $crate::api::logger::Logger::log($ctx, format_args!($($arg)*), $crate::logger::level::LogLevel::Info, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! debug {
    ($ctx:ident, $($arg:tt)*) => {{
        $crate::api::logger::Logger::log($ctx, format_args!($($arg)*), $crate::logger::level::LogLevel::Debug, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! warn {
    ($ctx:ident, $($arg:tt)*) => {{
        $crate::api::logger::Logger::log($ctx, format_args!($($arg)*), $crate::logger::level::LogLevel::Warning, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! error {
    ($ctx:ident, $($arg:tt)*) => {{
        $crate::api::logger::Logger::log($ctx, format_args!($($arg)*), $crate::logger::level::LogLevel::Error, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! panic {
    ($api:ident, $($arg:tt)*) => {{
        $crate::api::logger::Logger::log($ctx, format_args!($($arg)*), $crate::logger::level::LogLevel::Critical, Some((file!(), line!())));
        panic!()
    }}
}

#[macro_export]
macro_rules! expect {
    ($ctx:ident, $result:expr, $($arg:tt)*) => {{
        match $crate::api::logger::IntoResult::into_result($result) {
            std::result::Result::Err(_) => {
                $crate::api::logger::Logger::log($ctx, format_args!($($arg)*), $crate::logger::level::LogLevel::Critical, Some((file!(), line!())));
                panic!()
            },
            std::result::Result::Ok(v) => {
                v
            }
        }
    }};
    ($ctx:ident, $result:expr) => {{
        match $crate::api::logger::IntoResult::into_result($result) {
            std::result::Result::Err(e) => {
                $crate::api::logger::Logger::log($ctx, format_args!("{}", e), $crate::logger::level::LogLevel::Critical, Some((file!(), line!())));
                panic!()
            },
            std::result::Result::Ok(v) => {
                v
            }
        }
    }};
}

pub trait IntoResult<T, E> {
    fn into_result(self) -> Result<T, E>;
}

impl<T, E> IntoResult<T, E> for Result<T, E> {
    fn into_result(self) -> Result<T, E> {
        self
    }
}

impl<T> IntoResult<T, &'static str> for Option<T> {
    fn into_result(self) -> Result<T, &'static str> {
        match self {
            Some(v) => Ok(v),
            None => Err("Got value of None"),
        }
    }
}

pub struct Logger;

impl Logger {
    pub fn log(
        ctx: &Context,
        args: Arguments<'_>,
        level: LogLevel,
        source: Option<(&'static str, u32)>,
    ) {
        ctx.logger.log(args, level, source);
    }

    pub fn set_max_level(ctx: &mut Context, level: LogLevel) {
        ctx.logger.set_max_level(level);
    }
}
