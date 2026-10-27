#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("compile error: {0}")]
    Compile(#[source] color_eyre::Report),
    #[error("runtime error: {0}")]
    Runtime(#[source] color_eyre::Report),
}
