use std::fmt;

/// Crate-wide error type for download, query, and parsing operations.
///
/// Besides wrapping I/O, HTTP, JSON, CSV, and ZIP errors, it distinguishes caller mistakes
/// ([`DownloadError::InvalidQuery`]) from unexpected upstream or local data
/// ([`DownloadError::Data`]).
#[derive(Debug)]
#[non_exhaustive]
pub enum DownloadError {
    Io(std::io::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
    Csv(csv::Error),
    Zip(zip::result::ZipError),
    /// A query argument is malformed, unsupported, or outside the supported range.
    InvalidQuery(String),
    /// Downloaded or fetched data has an unexpected shape or content.
    Data(String),
}

impl DownloadError {
    pub(crate) fn invalid_query(message: impl Into<String>) -> Self {
        Self::InvalidQuery(message.into())
    }

    pub(crate) fn data(message: impl Into<String>) -> Self {
        Self::Data(message.into())
    }
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Http(err) => write!(f, "HTTP error: {err}"),
            Self::Json(err) => write!(f, "JSON parse error: {err}"),
            Self::Csv(err) => write!(f, "CSV parse error: {err}"),
            Self::Zip(err) => write!(f, "ZIP error: {err}"),
            Self::InvalidQuery(message) => write!(f, "invalid query: {message}"),
            Self::Data(message) => write!(f, "unexpected data: {message}"),
        }
    }
}

impl std::error::Error for DownloadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Http(err) => Some(err),
            Self::Json(err) => Some(err),
            Self::Csv(err) => Some(err),
            Self::Zip(err) => Some(err),
            Self::InvalidQuery(_) | Self::Data(_) => None,
        }
    }
}

impl From<std::io::Error> for DownloadError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<reqwest::Error> for DownloadError {
    fn from(err: reqwest::Error) -> Self {
        Self::Http(err)
    }
}

impl From<serde_json::Error> for DownloadError {
    fn from(err: serde_json::Error) -> Self {
        Self::Json(err)
    }
}

impl From<csv::Error> for DownloadError {
    fn from(err: csv::Error) -> Self {
        Self::Csv(err)
    }
}

impl From<zip::result::ZipError> for DownloadError {
    fn from(err: zip::result::ZipError) -> Self {
        Self::Zip(err)
    }
}
