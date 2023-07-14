use core::fmt::Display;

use crate::context::SystemContext;

pub trait SystemError: Display {}

pub type SystemResult = Result<(), Box<dyn SystemError>>;

pub type ExclusiveSystemCallback = fn(&mut SystemContext) -> SystemResult;
pub type ParallelSystemCallback = fn(&mut SystemContext) -> SystemResult;

impl SystemError for &str {}
impl SystemError for String {}
impl From<&str> for Box<dyn SystemError> {
    fn from(error: &str) -> Self {
        Box::new(error.to_string())
    }
}
