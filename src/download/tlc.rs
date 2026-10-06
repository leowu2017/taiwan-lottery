use std::fs;
use std::path::{Path, PathBuf};

use crate::DownloadError;

use super::common::{
    build_http_client, extract_zip_bytes, sanitize_file_name, should_extract_zip,
    zip_extract_dir_for_file,
};

const RESULT_DOWNLOAD_URL: &str = "https://api.taiwanlottery.com/TLCAPIWeB/Lottery/ResultDownload";
const FIRST_YEAR: i32 = 2007;

#[derive(Debug, serde::Deserialize)]
struct ResultDownloadResponse {
    #[serde(rename = "rtCode")]
    rt_code: i32,
    content: Option<ResultDownloadContent>,
}

#[derive(Debug, serde::Deserialize)]
struct ResultDownloadContent {
    #[serde(rename = "fileName")]
    file_name: Option<String>,
    path: Option<String>,
}

/// Resolves the yearly ZIP link. The API answers HTTP 500 for years without a file, so a
/// missing answer is only tolerated when `allow_missing` is set (the current year, which may
/// not be published yet); for earlier years it is reported instead of silently ending the download.
fn resolve_history_zip_for_year(
    client: &reqwest::blocking::Client,
    base_url: &str,
    year: i32,
    allow_missing: bool,
) -> Result<Option<ResultDownloadContent>, DownloadError> {
    let response = client.get(base_url).query(&[("year", year)]).send()?;

    if allow_missing && !response.status().is_success() {
        return Ok(None);
    }
    let response = response.error_for_status()?;

    let response_body = response.text()?;
    let parsed: ResultDownloadResponse = serde_json::from_str(&response_body)?;
    let content = parsed.content.filter(|content| {
        parsed.rt_code == 0
            && content
                .path
                .as_deref()
                .is_some_and(|path| !path.trim().is_empty())
    });

    match content {
        Some(content) => Ok(Some(content)),
        None if allow_missing => Ok(None),
        None => Err(DownloadError::data(format!(
            "Taiwan Lottery API has no history draw file for {year} (rtCode={})",
            parsed.rt_code
        ))),
    }
}

fn download_history_draw_with_client(
    client: &reqwest::blocking::Client,
    base_url: &str,
    last_year: i32,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, DownloadError> {
    fs::create_dir_all(output_dir)?;
    let code_dir = output_dir.join(super::gaze::HISTORY_DRAW_CODE);
    fs::create_dir_all(&code_dir)?;

    let mut saved_files = Vec::new();
    for year in FIRST_YEAR..=last_year {
        let metadata =
            match resolve_history_zip_for_year(client, base_url, year, year == last_year)? {
                Some(value) => value,
                None => continue,
            };

        let download_path = metadata
            .path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                DownloadError::data("Taiwan Lottery API returned empty download path")
            })?;

        let file_bytes = client
            .get(download_path)
            .send()?
            .error_for_status()?
            .bytes()?;

        let mut file_name = metadata
            .file_name
            .as_deref()
            .map(sanitize_file_name)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| year.to_string());
        if file_name.rsplit_once('.').is_none() {
            file_name.push_str(".zip");
        }

        let out_path = code_dir.join(&file_name);
        fs::write(&out_path, &file_bytes)?;
        saved_files.push(out_path.clone());

        if should_extract_zip(&file_name, &file_bytes) {
            let extract_dir = zip_extract_dir_for_file(&code_dir, &file_name, year as usize);
            let extracted_files = extract_zip_bytes(&file_bytes, &extract_dir)?;
            saved_files.extend(extracted_files);
        }
    }

    if saved_files.is_empty() {
        return Err(DownloadError::data(
            "no downloadable history draw zip in Taiwan Lottery API",
        ));
    }

    Ok(saved_files)
}

pub fn download_history_draw(output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    let current_year = time::OffsetDateTime::now_utc().year();
    download_history_draw_with_client(&client, RESULT_DOWNLOAD_URL, current_year, output_dir)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{zip_bytes, MockResponse, MockServer};

    fn temp_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("taiwan-lottery-tlc-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn year_server(failing_years: &'static [i32]) -> MockServer {
        let zip = zip_bytes(&[("lotto.csv", b"a,b\n1,2\n".as_slice())]);
        let base = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
        let base_for_handler = std::sync::Arc::clone(&base);
        let server = MockServer::start(move |target| {
            if target.starts_with("/zip") {
                return MockResponse::ok(zip.clone());
            }
            let year: i32 = target
                .split("year=")
                .nth(1)
                .and_then(|value| value.parse().ok())
                .unwrap_or(0);
            if failing_years.contains(&year) {
                return MockResponse::status(500);
            }
            let body = format!(
                r#"{{"rtCode":0,"content":{{"fileName":"{year}","path":"{}/zip/{year}"}}}}"#,
                base_for_handler.lock().expect("lock")
            );
            MockResponse::ok(body)
        });
        *base.lock().expect("lock") = server.base_url.clone();
        server
    }

    #[test]
    fn missing_past_year_is_reported_instead_of_ending_silently() {
        let server = year_server(&[2007]);
        let dir = temp_dir("missing-past");
        let client = build_http_client().expect("client");

        let result = download_history_draw_with_client(&client, &server.base_url, 2008, &dir);
        assert!(matches!(result, Err(DownloadError::Http(_))));
        assert_eq!(server.hits(), 1, "must stop at the first failing year");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unpublished_current_year_is_skipped() {
        let server = year_server(&[2008]);
        let dir = temp_dir("missing-current");
        let client = build_http_client().expect("client");

        let files = download_history_draw_with_client(&client, &server.base_url, 2008, &dir)
            .expect("current year may be absent");
        assert!(files.iter().any(|path| path.ends_with("2007.zip")));
        assert!(!files.iter().any(|path| path.ends_with("2008.zip")));

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn all_years_are_downloaded_and_extracted() {
        let server = year_server(&[]);
        let dir = temp_dir("all-years");
        let client = build_http_client().expect("client");

        let files = download_history_draw_with_client(&client, &server.base_url, 2008, &dir)
            .expect("download");
        for year in ["2007", "2008"] {
            assert!(files
                .iter()
                .any(|path| path.ends_with(format!("{year}.zip"))));
        }
        assert_eq!(
            files
                .iter()
                .filter(|path| path.ends_with("lotto.csv"))
                .count(),
            2
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
