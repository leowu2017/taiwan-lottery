use std::collections::HashMap;
use std::ffi::CStr;
use std::fs;
use std::os::raw::c_char;
use std::path::{Path, PathBuf};
use std::time::Duration;

const CSV_BASE_URL: &str = "https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=";
const API_DOCS_URL: &str = "https://gaze.nta.gov.tw/ntaOpenApi/v2/api-docs?group=FinancialPlanning";
const API_DOCS_FILE_NAME: &str = "financialplanning_api_docs.json";

#[derive(Debug, serde::Deserialize)]
struct ApiDocs {
    paths: HashMap<String, serde_json::Value>,
}

#[derive(Debug)]
pub enum DownloadError {
    Io(std::io::Error),
    Http(reqwest::Error),
    Json(serde_json::Error),
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

pub fn build_csv_url(code: &str) -> String {
    format!("{CSV_BASE_URL}{code}")
}

pub fn parse_codes_from_api_docs(api_docs_json: &str) -> Result<Vec<String>, DownloadError> {
    let api_docs: ApiDocs = serde_json::from_str(api_docs_json)?;
    let mut codes: Vec<String> = api_docs
        .paths
        .keys()
        .filter_map(|path| path.strip_prefix("/restful/"))
        .filter(|code| !code.is_empty())
        .map(ToOwned::to_owned)
        .collect();

    codes.sort();
    codes.dedup();
    Ok(codes)
}

pub fn download_all_data(output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    fs::create_dir_all(output_dir)?;

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()?;

    let api_docs_body = client
        .get(API_DOCS_URL)
        .send()?
        .error_for_status()?
        .text()?;

    let api_docs_out_path = output_dir.join(API_DOCS_FILE_NAME);
    fs::write(&api_docs_out_path, api_docs_body.as_bytes())?;

    let codes = parse_codes_from_api_docs(&api_docs_body)?;

    let mut saved_files = Vec::with_capacity(codes.len() + 1);
    saved_files.push(api_docs_out_path);
    for code in codes {
        let url = build_csv_url(&code);
        let body = client.get(&url).send()?.error_for_status()?.bytes()?;

        let out_path = output_dir.join(format!("{code}.csv"));
        fs::write(&out_path, &body)?;
        saved_files.push(out_path);
    }

    Ok(saved_files)
}

#[repr(i32)]
enum DownloadAllDataStatus {
    Success = 0,
    NullPath = 1,
    InvalidPathUtf8 = 2,
    Io = 3,
    Network = 4,
    Parse = 5,
}

#[unsafe(export_name = "download_all_data")]
pub extern "C" fn download_all_data_ffi(output_dir: *const c_char) -> i32 {
    if output_dir.is_null() {
        return DownloadAllDataStatus::NullPath as i32;
    }

    // SAFETY: output_dir is checked for null above and expected to point to a valid C string.
    let c_str = unsafe { CStr::from_ptr(output_dir) };
    let out_dir = match c_str.to_str() {
        Ok(path) => path,
        Err(_) => return DownloadAllDataStatus::InvalidPathUtf8 as i32,
    };

    match download_all_data(out_dir) {
        Ok(_) => DownloadAllDataStatus::Success as i32,
        Err(DownloadError::Io(_)) => DownloadAllDataStatus::Io as i32,
        Err(DownloadError::Http(_)) => DownloadAllDataStatus::Network as i32,
        Err(DownloadError::Json(_)) => DownloadAllDataStatus::Parse as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_codes_from_api_docs_works() {
        let sample = r#"
        {
            "paths": {
                "/restful/D423F": {},
                "/restful/D401": {},
                "/health": {}
            }
        }
        "#;

        let codes = parse_codes_from_api_docs(sample).expect("must parse codes");
        assert_eq!(codes, vec!["D401".to_string(), "D423F".to_string()]);
    }

    #[test]
    fn csv_url_is_built_correctly() {
        assert_eq!(
            build_csv_url("D423F"),
            "https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=D423F"
        );
    }
}
