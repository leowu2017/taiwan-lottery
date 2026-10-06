use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use encoding_rs::BIG5;
use reqwest::header::{CONTENT_DISPOSITION, CONTENT_TYPE};

use crate::DownloadError;

/// Upper bound for a single downloaded file body.
pub(crate) const MAX_DOWNLOAD_BYTES: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ExtractLimits {
    pub(crate) max_entries: usize,
    pub(crate) max_total_bytes: u64,
}

pub(crate) const DEFAULT_EXTRACT_LIMITS: ExtractLimits = ExtractLimits {
    max_entries: 10_000,
    max_total_bytes: 1024 * 1024 * 1024,
};

const WINDOWS_RESERVED_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Reads a response body, failing instead of buffering more than `limit` bytes.
pub(crate) fn read_body_limited(
    response: reqwest::blocking::Response,
    limit: u64,
) -> Result<Vec<u8>, DownloadError> {
    if response
        .content_length()
        .is_some_and(|length| length > limit)
    {
        return Err(DownloadError::data(format!(
            "download exceeds the {limit} byte limit"
        )));
    }

    let mut body = Vec::new();
    response.take(limit + 1).read_to_end(&mut body)?;
    if body.len() as u64 > limit {
        return Err(DownloadError::data(format!(
            "download exceeds the {limit} byte limit"
        )));
    }

    Ok(body)
}

pub(crate) fn sanitize_file_name(file_name: &str) -> String {
    let invalid_chars = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut sanitized: String = file_name
        .trim()
        .chars()
        .map(|ch| {
            if ch.is_control() || invalid_chars.contains(&ch) {
                '_'
            } else {
                ch
            }
        })
        .collect();

    while sanitized.ends_with('.') || sanitized.ends_with(' ') {
        sanitized.pop();
    }

    let stem = sanitized.split('.').next().unwrap_or_default();
    if WINDOWS_RESERVED_NAMES
        .iter()
        .any(|reserved| reserved.eq_ignore_ascii_case(stem))
    {
        sanitized.insert(0, '_');
    }

    if sanitized.is_empty() {
        "download.bin".to_string()
    } else {
        sanitized
    }
}

pub(crate) fn file_name_from_content_disposition(header_value: &str) -> Option<String> {
    for part in header_value.split(';').map(str::trim) {
        if let Some(raw_value) = part.strip_prefix("filename*=") {
            let raw_value = raw_value.trim_matches('"');
            if let Some(file_name) = decode_rfc5987_filename(raw_value) {
                return Some(file_name);
            }
        }
    }

    for part in header_value.split(';').map(str::trim) {
        if let Some(raw_value) = part.strip_prefix("filename=") {
            let decoded = decode_filename_value(raw_value.trim_matches('"'));
            let file_name = sanitize_file_name(&decoded);
            if !file_name.is_empty() {
                return Some(file_name);
            }
        }
    }

    None
}

pub(crate) fn header_value_to_text(value: &reqwest::header::HeaderValue) -> String {
    if let Ok(text) = value.to_str() {
        return text.to_string();
    }

    value.as_bytes().iter().map(|b| *b as char).collect()
}

fn decode_rfc5987_filename(raw_value: &str) -> Option<String> {
    let (charset, encoded_value) = if let Some((charset, rest)) = raw_value.split_once('\'') {
        if let Some((_, value)) = rest.split_once('\'') {
            (charset, value)
        } else {
            ("utf-8", raw_value)
        }
    } else {
        ("utf-8", raw_value)
    };

    let bytes = percent_encoding::percent_decode_str(encoded_value).collect::<Vec<u8>>();
    let decoded = if charset.eq_ignore_ascii_case("utf-8") || charset.eq_ignore_ascii_case("utf8") {
        String::from_utf8(bytes).ok()
    } else if charset.eq_ignore_ascii_case("big5") || charset.eq_ignore_ascii_case("cp950") {
        let (text, _, had_errors) = BIG5.decode(&bytes);
        if had_errors {
            None
        } else {
            Some(text.into_owned())
        }
    } else {
        String::from_utf8(bytes).ok()
    };

    decoded.and_then(|value| {
        let file_name = sanitize_file_name(&value);
        if file_name.is_empty() {
            None
        } else {
            Some(file_name)
        }
    })
}

fn decode_filename_value(raw_value: &str) -> String {
    let raw_value = raw_value.trim();

    if raw_value.contains('%') {
        let bytes = percent_encoding::percent_decode_str(raw_value).collect::<Vec<u8>>();
        if let Ok(utf8) = String::from_utf8(bytes.clone()) {
            return utf8;
        }

        let (big5_text, _, had_errors) = BIG5.decode(&bytes);
        if !had_errors {
            return big5_text.into_owned();
        }
    }

    if let Some(fixed) = try_fix_latin1_utf8_mojibake(raw_value) {
        return fixed;
    }

    raw_value.to_string()
}

fn try_fix_latin1_utf8_mojibake(value: &str) -> Option<String> {
    if !value.chars().all(|ch| ch <= '\u{00FF}') {
        return None;
    }

    let bytes: Vec<u8> = value.chars().map(|ch| ch as u8).collect();
    let decoded = String::from_utf8(bytes).ok()?;
    if decoded == value {
        None
    } else {
        Some(decoded)
    }
}

fn extension_from_content_type(headers: &reqwest::header::HeaderMap) -> Option<&'static str> {
    let content_type = headers.get(CONTENT_TYPE)?.to_str().ok()?;
    let mime = content_type
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or(content_type);

    match mime {
        "application/pdf" => Some("pdf"),
        "application/zip" => Some("zip"),
        "application/msword" => Some("doc"),
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document" => Some("docx"),
        "application/vnd.ms-excel" => Some("xls"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Some("xlsx"),
        "application/vnd.ms-powerpoint" => Some("ppt"),
        "application/vnd.openxmlformats-officedocument.presentationml.presentation" => Some("pptx"),
        "text/csv" => Some("csv"),
        "text/plain" => Some("txt"),
        "image/jpeg" => Some("jpg"),
        "image/png" => Some("png"),
        _ => None,
    }
}

pub(crate) fn extension_from_magic_bytes(file_bytes: &[u8]) -> Option<&'static str> {
    if file_bytes.starts_with(b"%PDF-") {
        return Some("pdf");
    }
    if file_bytes.starts_with(&[0x50, 0x4B, 0x03, 0x04]) {
        return Some("zip");
    }
    if file_bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        return Some("png");
    }
    if file_bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return Some("jpg");
    }

    None
}

pub(crate) fn should_extract_zip(file_name: &str, file_bytes: &[u8]) -> bool {
    file_name.to_ascii_lowercase().ends_with(".zip")
        || matches!(extension_from_magic_bytes(file_bytes), Some("zip"))
}

pub(crate) fn zip_extract_dir_for_file(
    code_dir: &Path,
    file_name: &str,
    fallback_index: usize,
) -> PathBuf {
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .map(sanitize_file_name)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("archive_{fallback_index}"));

    code_dir.join(stem)
}

pub(crate) fn decode_non_utf8_zip_name(raw_name: &[u8], fallback_name: &str) -> String {
    let (decoded, _, had_errors) = BIG5.decode(raw_name);
    if had_errors {
        fallback_name.to_string()
    } else {
        decoded.into_owned()
    }
}

pub(crate) fn normalize_zip_entry_path(entry_name: &str) -> Option<PathBuf> {
    let mut path = PathBuf::new();
    for component in entry_name.split(['/', '\\']) {
        let trimmed = component.trim();
        if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
            continue;
        }

        let clean = sanitize_file_name(trimmed);
        if clean.is_empty() {
            continue;
        }
        path.push(clean);
    }

    if path.as_os_str().is_empty() {
        None
    } else {
        Some(path)
    }
}

pub(crate) fn extract_zip_bytes(
    zip_bytes: &[u8],
    extract_dir: &Path,
) -> Result<Vec<PathBuf>, DownloadError> {
    extract_zip_bytes_with_limits(zip_bytes, extract_dir, DEFAULT_EXTRACT_LIMITS)
}

/// Picks a path not used yet in this archive; names that collide after sanitizing or that
/// differ only by case get a numeric suffix instead of overwriting each other.
fn unique_out_path(path: PathBuf, used: &mut HashSet<String>) -> PathBuf {
    let key = |candidate: &Path| candidate.to_string_lossy().to_lowercase();
    if used.insert(key(&path)) {
        return path;
    }

    let stem = path
        .file_stem()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default();
    let extension = path
        .extension()
        .map(|value| format!(".{}", value.to_string_lossy()))
        .unwrap_or_default();

    let mut suffix = 2usize;
    loop {
        let candidate = path.with_file_name(format!("{stem}_{suffix}{extension}"));
        if used.insert(key(&candidate)) {
            return candidate;
        }
        suffix += 1;
    }
}

pub(crate) fn extract_zip_bytes_with_limits(
    zip_bytes: &[u8],
    extract_dir: &Path,
    limits: ExtractLimits,
) -> Result<Vec<PathBuf>, DownloadError> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes))?;
    if archive.len() > limits.max_entries {
        return Err(DownloadError::data(format!(
            "zip archive has more than {} entries",
            limits.max_entries
        )));
    }
    fs::create_dir_all(extract_dir)?;

    let mut extracted_files = Vec::new();
    let mut used_paths = HashSet::new();
    let mut remaining = limits.max_total_bytes;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let entry_name = if let Ok(utf8) = std::str::from_utf8(entry.name_raw()) {
            utf8.to_string()
        } else {
            decode_non_utf8_zip_name(entry.name_raw(), entry.name())
        };

        let Some(relative_path) = normalize_zip_entry_path(&entry_name) else {
            continue;
        };

        let out_path = extract_dir.join(relative_path);
        if entry.is_dir() {
            fs::create_dir_all(&out_path)?;
            continue;
        }

        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let out_path = unique_out_path(out_path, &mut used_paths);
        let mut out_file = fs::File::create(&out_path)?;
        // The declared size can lie, so the copy itself is bounded too.
        let written = std::io::copy(&mut (&mut entry).take(remaining + 1), &mut out_file)?;
        if written > remaining {
            drop(out_file);
            let _ = fs::remove_file(&out_path);
            return Err(DownloadError::data(format!(
                "zip content exceeds the {} byte extraction limit",
                limits.max_total_bytes
            )));
        }
        remaining -= written;
        extracted_files.push(out_path);
    }

    Ok(extracted_files)
}

pub(crate) fn pick_download_file_name(
    link: &str,
    headers: &reqwest::header::HeaderMap,
    fallback_index: usize,
) -> String {
    let mut file_name = if let Some(content_disposition) = headers.get(CONTENT_DISPOSITION) {
        let value_text = header_value_to_text(content_disposition);
        file_name_from_content_disposition(&value_text)
    } else {
        None
    }
    .or_else(|| {
        link.split('?')
            .next()
            .and_then(|path| path.rsplit('/').next())
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(sanitize_file_name)
    })
    .unwrap_or_else(|| format!("download_{fallback_index}.bin"));

    if file_name.rsplit_once('.').is_none() {
        if let Some(extension) = extension_from_content_type(headers) {
            file_name.push('.');
            file_name.push_str(extension);
        }
    }

    file_name
}

pub(crate) fn build_http_client() -> Result<reqwest::blocking::Client, DownloadError> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(DownloadError::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{zip_bytes, MockResponse, MockServer};

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "taiwan-lottery-common-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn sanitize_file_name_prefixes_windows_reserved_names() {
        assert_eq!(sanitize_file_name("CON.csv"), "_CON.csv");
        assert_eq!(sanitize_file_name("nul"), "_nul");
        assert_eq!(sanitize_file_name("Com1.txt"), "_Com1.txt");
        assert_eq!(sanitize_file_name("console.csv"), "console.csv");
    }

    #[test]
    fn extraction_keeps_entries_that_differ_only_by_case() {
        let archive = zip_bytes(&[
            ("a.csv", b"first".as_slice()),
            ("A.csv", b"second".as_slice()),
        ]);
        let dir = temp_dir("collision");

        let files = extract_zip_bytes(&archive, &dir).expect("extract");
        assert_eq!(files.len(), 2);
        let mut contents: Vec<String> = files
            .iter()
            .map(|path| fs::read_to_string(path).expect("read"))
            .collect();
        contents.sort();
        assert_eq!(contents, vec!["first", "second"]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn extraction_rejects_archives_over_the_size_limit() {
        let archive = zip_bytes(&[("big.csv", [b'x'; 100].as_slice())]);
        let dir = temp_dir("size-limit");
        let limits = ExtractLimits {
            max_entries: 10,
            max_total_bytes: 50,
        };

        let err = extract_zip_bytes_with_limits(&archive, &dir, limits).expect_err("too big");
        assert!(matches!(err, DownloadError::Data(_)));
        assert!(!dir.join("big.csv").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn extraction_rejects_archives_with_too_many_entries() {
        let archive = zip_bytes(&[("a.csv", b"1".as_slice()), ("b.csv", b"2".as_slice())]);
        let dir = temp_dir("entry-limit");
        let limits = ExtractLimits {
            max_entries: 1,
            max_total_bytes: 1024,
        };

        let err = extract_zip_bytes_with_limits(&archive, &dir, limits).expect_err("too many");
        assert!(matches!(err, DownloadError::Data(_)));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_body_limited_rejects_oversized_bodies() {
        let server = MockServer::start(|_| MockResponse::ok(vec![b'x'; 64]));
        let client = build_http_client().expect("client");

        let within = client.get(&server.base_url).send().expect("send");
        assert_eq!(read_body_limited(within, 64).expect("fits").len(), 64);

        let over = client.get(&server.base_url).send().expect("send");
        let err = read_body_limited(over, 63).expect_err("too large");
        assert!(matches!(err, DownloadError::Data(_)));
    }
}
