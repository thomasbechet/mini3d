#[macro_export]
macro_rules! info {
    ($api:ident, $($arg:tt)*) => {{
        $api.logger.log(format_args!($($arg)*), $crate::logger::level::LogLevel::Info, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! debug {
    ($api:ident, $($arg:tt)*) => {{
        $api.logger.log(format_args!($($arg)*), $crate::logger::level::LogLevel::Debug, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! warn {
    ($api:ident, $($arg:tt)*) => {{
        $api.logger.log(format_args!($($arg)*), $crate::logger::level::LogLevel::Warning, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! error {
    ($api:ident, $($arg:tt)*) => {{
        $api.logger.log(format_args!($($arg)*), $crate::logger::level::LogLevel::Error, Some((file!(), line!())));
    }}
}

#[macro_export]
macro_rules! panic {
    ($api:ident, $($arg:tt)*) => {{
        $api.logger.log(format_args!($($arg)*), $crate::logger::level::LogLevel::Critical, Some((file!(), line!())));
        panic!()
    }}
}

#[macro_export]
macro_rules! expect {
    ($api:ident, $result:expr, $($arg:tt)*) => {{
        $crate::ecs::api::logger::IntoResult::into_result($result).unwrap_or_else(|_| {
            $api.logger.log(format_args!($($arg)*), $crate::logger::level::LogLevel::Critical, Some((file!(), line!())));
            panic!()
        })
    }};
    ($api:ident, $result:expr) => {{
        $crate::ecs::api::logger::IntoResult::into_result($result).unwrap_or_else(|e| {
            $api.logger.log(format_args!("{}", e), $crate::logger::level::LogLevel::Critical, Some((file!(), line!())));
            panic!()
        })
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
