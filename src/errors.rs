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
