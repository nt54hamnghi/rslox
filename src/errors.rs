#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Compile(#[source] color_eyre::Report),
    #[error("{0}")]
    Runtime(#[source] color_eyre::Report),
}

impl From<crate::compiler::Error> for Error {
    fn from(err: crate::compiler::Error) -> Self {
        Self::Compile(color_eyre::Report::from(err))
    }
}
