use std::fmt::Display;

#[derive(Debug, thiserror::Error)]
pub struct Report {
    line: u32,
    location: Option<String>,
    message: String,
}

impl Report {
    pub fn error(line: u32, message: String) -> Self {
        Self {
            line,
            location: None,
            message,
        }
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let location = self.location.as_deref().unwrap_or_default();
        write!(
            f,
            "[line {}] Error{}: {}",
            self.line, location, self.message
        )
    }
}
