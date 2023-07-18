use core::fmt::Display;

pub trait SystemError: Display {}

pub type SystemResult = Result<(), Box<dyn SystemError>>;

impl SystemError for &str {}
impl SystemError for String {}
impl From<&str> for Box<dyn SystemError> {
    fn from(error: &str) -> Self {
        Box::new(error.to_string())
    }
}
