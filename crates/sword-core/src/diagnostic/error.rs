use super::Diagnostic;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};

/// The error type returned by fallible Sword APIs.
///
/// It owns a [`Diagnostic`] and, when the failure came from something else, the
/// original error. The diagnostic is what gets rendered, the original error is
/// exposed through [`std::error::Error::source`] so it can still be inspected.
#[derive(Debug)]
pub struct Error {
    diagnostic: Box<Diagnostic>,
    source: Option<Box<dyn StdError + Send + Sync>>,
}

impl Error {
    pub fn new(diagnostic: Diagnostic) -> Self {
        Self {
            diagnostic: Box::new(diagnostic),
            source: None,
        }
    }

    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    pub fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }

    pub fn report(self) {
        self.diagnostic.report();
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.diagnostic.fmt(f)
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source
            .as_deref()
            .map(|source| source as &(dyn StdError + 'static))
    }
}
