use std::{error::Error, fmt::{Display, Debug}};

pub struct ErrorWithContext {
    error: Box<dyn Error>,
    context: Box<dyn Display + 'static>,
}

impl ErrorWithContext {

    fn new<E, C>(error: E, context: C) -> Self
        where E: Error + 'static,
              C: Display + 'static {
        Self {
            error: Box::new(error),
            context: Box::new(context),
        }
    }
}

impl Error for ErrorWithContext {}

impl Debug for ErrorWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.context, self.error)
    }
}

impl Display for ErrorWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.context, self.error)
    }
}

pub trait ContextError<T, E: Error> {
    fn with_context<C, F>(self, f: F) -> Result<T, ErrorWithContext>
        where C: Display + 'static,
              F: FnOnce() -> C;
}

impl<T, E: Error + 'static> ContextError<T, E> for Result<T, E> {
    fn with_context<C, F>(self, f: F) -> Result<T, ErrorWithContext>
        where C: Display + 'static,
              F: FnOnce() -> C {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(ErrorWithContext::new(error, f())),
        }
    }
}