#[derive(Debug, thiserror::Error)]
#[error("[line {line}] Error: {message}")]
pub struct Report {
    line: u32,
    _location: Option<String>,
    message: String,
}

impl Report {
    pub fn error(line: u32, message: String) -> Self {
        Self {
            line,
            _location: None,
            message,
        }
    }
}
