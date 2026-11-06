#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Compile(#[source] color_eyre::Report),
    #[error("{0}")]
    Runtime(#[source] color_eyre::Report),
}

impl Error {
    pub fn compile<E>(err: E) -> Self
    where
        E: Into<color_eyre::Report>,
    {
        Error::Compile(err.into())
    }

    pub fn runtime<E>(err: E) -> Self
    where
        E: Into<color_eyre::Report>,
    {
        Error::Runtime(err.into())
    }
}
