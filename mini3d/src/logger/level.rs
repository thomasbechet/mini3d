use std::fmt::Display;

#[derive(Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    #[default]
    Info = 4,
    Debug = 3,
    Warning = 2,
    Error = 1,
    Critical = 0,
}
