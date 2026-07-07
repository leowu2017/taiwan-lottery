use std::fmt;

/// Error types that can occur during download, parsing, or API operations.
///
/// This enum wraps errors from various operations including file I/O, HTTP requests,
/// JSON parsing, CSV parsing, and ZIP extraction.
#[derive(Debug)]
pub enum DownloadError {
    Io(std::io::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
    Csv(csv::Error),
    Zip(zip::result::ZipError),
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Http(err) => write!(f, "HTTP error: {err}"),
            Self::Json(err) => write!(f, "JSON parse error: {err}"),
            Self::Csv(err) => write!(f, "CSV parse error: {err}"),
            Self::Zip(err) => write!(f, "ZIP error: {err}"),
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
