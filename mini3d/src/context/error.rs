use core::{result::Result, fmt::{Display, Formatter}};

use crate::ecs::system::SystemError;

pub struct ErrorWithContext {
    error: Box<dyn SystemError>,
    context: Box<dyn Display>,
}

impl ErrorWithContext {
    fn new<E, C>(error: E, context: C) -> Self
        where E: SystemError + 'static,
              C: Display + 'static {
        Self {
            error: Box::new(error),
            context: Box::new(context),
        }
    }
}

impl Display for ErrorWithContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}: {}", self.context, self.error)
    }
}

impl SystemError for ErrorWithContext {}

impl From<ErrorWithContext> for Box<dyn SystemError> {
    fn from(error: ErrorWithContext) -> Self {
        Box::new(error)
    }
}

pub trait ContextError<T, E> {
    fn with_context<C, F>(self, f: F) -> Result<T, ErrorWithContext>
        where C: Display + 'static,
              F: FnOnce() -> C;
}

impl<T, E: SystemError + 'static> ContextError<T, E> for Result<T, E> {
    fn with_context<C, F>(self, f: F) -> Result<T, ErrorWithContext>
        where C: Display + 'static,
              F: FnOnce() -> C {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(ErrorWithContext::new(error, f())),
        }
    }
}