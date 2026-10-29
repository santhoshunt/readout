use serde::ser::SerializeStruct;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("file not found: {0}")]
    NotFound(String),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("settings: {0}")]
    Settings(String),
    #[error("editor: {0}")]
    Editor(String),
    /// The file changed on disk since the caller last read it, so the write was refused.
    #[error("{0}")]
    Conflict(String),
    /// Readout won't write this file (not UTF-8, too large, or editing is switched off).
    #[error("{0}")]
    ReadOnly(String),
}

pub type AppResult<T> = Result<T, AppError>;

// The front end receives { kind, message } so it can pick a status-bar text by kind.
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let kind = match self {
            AppError::NotFound(_) => "NotFound",
            AppError::Io(_) => "Io",
            AppError::Settings(_) => "Settings",
            AppError::Editor(_) => "Editor",
            AppError::Conflict(_) => "Conflict",
            AppError::ReadOnly(_) => "ReadOnly",
        };
        let mut st = s.serialize_struct("AppError", 2)?;
        st.serialize_field("kind", kind)?;
        st.serialize_field("message", &self.to_string())?;
        st.end()
    }
}
