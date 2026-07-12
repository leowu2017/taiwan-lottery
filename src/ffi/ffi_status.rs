use crate::DownloadError;

#[repr(i32)]
pub(crate) enum DownloadStatus {
    Success = 0,
    NullPath = 1,
    InvalidPathUtf8 = 2,
    Io = 3,
    Network = 4,
    Parse = 5,
    NullDatasetCode = 6,
    InvalidDatasetCodeUtf8 = 7,
    InvalidGame = 8,
    InvalidQueryUtf8 = 9,
    NullResultPointer = 10,
    InvalidLanguage = 11,
}

pub(crate) fn map_download_result<T>(result: Result<T, DownloadError>) -> i32 {
    match result {
        Ok(_) => DownloadStatus::Success as i32,
        Err(DownloadError::Io(_)) => DownloadStatus::Io as i32,
        Err(DownloadError::Http(_)) => DownloadStatus::Network as i32,
        Err(DownloadError::Json(_) | DownloadError::Csv(_) | DownloadError::Zip(_)) => {
            DownloadStatus::Parse as i32
        }
    }
}
