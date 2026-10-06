use std::error::Error as _;
use taiwan_lottery::DownloadError;

#[test]
fn display_formats_io_error() {
    let err = DownloadError::Io(std::io::Error::other("disk full"));
    assert_eq!(err.to_string(), "I/O error: disk full");
}

#[test]
fn source_returns_wrapped_error() {
    let err = DownloadError::Io(std::io::Error::other("disk full"));
    let source = err.source().expect("wrapped error source");
    assert_eq!(source.to_string(), "disk full");
}

#[test]
fn display_formats_invalid_query_and_data_errors() {
    let invalid = DownloadError::InvalidQuery("month must be between 01 and 12".to_string());
    assert_eq!(
        invalid.to_string(),
        "invalid query: month must be between 01 and 12"
    );
    assert!(invalid.source().is_none());

    let data = DownloadError::Data("missing column".to_string());
    assert_eq!(data.to_string(), "unexpected data: missing column");
    assert!(data.source().is_none());
}
