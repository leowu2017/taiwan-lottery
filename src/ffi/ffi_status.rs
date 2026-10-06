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
    InvalidQuery = 12,
}

pub(crate) fn map_download_result<T>(result: Result<T, DownloadError>) -> i32 {
    match result {
        Ok(_) => DownloadStatus::Success as i32,
        Err(err) => status_for_error(&err) as i32,
    }
}

pub(crate) fn status_for_error(err: &DownloadError) -> DownloadStatus {
    match err {
        DownloadError::Io(_) => DownloadStatus::Io,
        DownloadError::Http(_) => DownloadStatus::Network,
        DownloadError::Json(_)
        | DownloadError::Csv(_)
        | DownloadError::Zip(_)
        | DownloadError::Data(_) => DownloadStatus::Parse,
        DownloadError::InvalidQuery(_) => DownloadStatus::InvalidQuery,
    }
}
