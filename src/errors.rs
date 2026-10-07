#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("compile error")]
    Compile,
    #[error("runtime error")]
    Runtime,
}
