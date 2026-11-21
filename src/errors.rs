use std::{error, fmt};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Compile(#[source] color_eyre::Report),
    #[error("{0}")]
    Runtime(#[source] color_eyre::Report),
}

pub trait LocationContext<E: error::Error> {
    fn at(self, line: usize) -> Located<E>;
    fn at_site(self, line: usize, site: impl Into<String>) -> Located<E>;
}

/// An error annotated with the source line and, optionally,
/// the site where it occurred.
///
/// Its display representation is `[line N] error: message`, or
/// `[line N] error at site: message` when a site is available.
#[derive(Debug, thiserror::Error)]
pub struct Located<E: error::Error> {
    inner: E,
    line: usize,
    site: Option<String>,
}

impl<E: error::Error> fmt::Display for Located<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] error", self.line)?;
        if let Some(span) = &self.site {
            write!(f, " at {span}")?;
        }
        write!(f, ": {}", self.inner)
    }
}

impl<E: error::Error> Located<E> {
    fn new(inner: E, line: usize, site: Option<String>) -> Self {
        Self { inner, line, site }
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0}")]
pub struct MessageError(String);

impl MessageError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl<E: error::Error> LocationContext<E> for E {
    fn at(self, line: usize) -> Located<E> {
        Located::new(self, line, None)
    }

    fn at_site(self, line: usize, site: impl Into<String>) -> Located<E> {
        let site = format!("'{}'", site.into());
        Located::new(self, line, Some(site))
    }
}

impl LocationContext<MessageError> for String {
    fn at(self, line: usize) -> Located<MessageError> {
        MessageError::new(self).at(line)
    }

    fn at_site(self, line: usize, site: impl Into<String>) -> Located<MessageError> {
        MessageError::new(self).at_site(line, site)
    }
}

impl LocationContext<MessageError> for &str {
    fn at(self, line: usize) -> Located<MessageError> {
        MessageError::new(self).at(line)
    }

    fn at_site(self, line: usize, site: impl Into<String>) -> Located<MessageError> {
        MessageError::new(self).at_site(line, site)
    }
}
