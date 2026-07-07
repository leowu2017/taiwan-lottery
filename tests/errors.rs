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
