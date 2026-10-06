use std::panic::{catch_unwind, AssertUnwindSafe};

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
    InternalError = 13,
}

/// Runs an FFI body and converts a panic into a status code so it never unwinds into C.
pub(crate) fn guard(body: impl FnOnce() -> i32) -> i32 {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or(DownloadStatus::InternalError as i32)
}

/// Same as [`guard`] for FFI functions that return nothing; a panic is swallowed.
pub(crate) fn guard_void(body: impl FnOnce()) {
    let _ = catch_unwind(AssertUnwindSafe(body));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn guard_converts_panic_into_internal_error() {
        let status = guard(|| panic!("boom"));
        assert_eq!(status, DownloadStatus::InternalError as i32);
    }

    #[test]
    fn guard_passes_through_normal_status() {
        assert_eq!(guard(|| 7), 7);
    }

    #[test]
    fn guard_void_swallows_panic() {
        guard_void(|| panic!("boom"));
    }

    #[test]
    fn errors_map_to_distinct_statuses() {
        let invalid = DownloadError::InvalidQuery("bad".to_string());
        let data = DownloadError::Data("bad".to_string());
        let io = DownloadError::Io(std::io::Error::other("bad"));
        assert_eq!(map_download_result::<()>(Err(invalid)), 12);
        assert_eq!(map_download_result::<()>(Err(data)), 5);
        assert_eq!(map_download_result::<()>(Err(io)), 3);
    }
}
