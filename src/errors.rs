#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Compile(#[source] color_eyre::Report),
    #[error("{0}")]
    Runtime(#[source] color_eyre::Report),
}
