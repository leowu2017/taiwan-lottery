use std::fs;
use std::path::{Path, PathBuf};

use crate::DownloadError;

use super::common::{
    build_http_client, extract_zip_bytes, sanitize_file_name, should_extract_zip,
    zip_extract_dir_for_file,
};

const RESULT_DOWNLOAD_URL: &str = "https://api.taiwanlottery.com/TLCAPIWeB/Lottery/ResultDownload";
const FALLBACK_START_YEAR: i32 = 2007;
const FALLBACK_MAX_YEAR: i32 = 2200;

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

fn resolve_history_zip_for_year(
    client: &reqwest::blocking::Client,
    year: i32,
) -> Result<Option<ResultDownloadContent>, DownloadError> {
    let response = client
        .get(RESULT_DOWNLOAD_URL)
        .query(&[("year", year)])
        .send()?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let response_body = response.text()?;
    let parsed: ResultDownloadResponse = serde_json::from_str(&response_body)?;
    if parsed.rt_code != 0 {
        return Ok(None);
    }

    let Some(content) = parsed.content else {
        return Ok(None);
    };
    let has_path = content
        .path
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .is_some();

    if has_path {
        Ok(Some(content))
    } else {
        Ok(None)
    }
}

fn download_history_draw_with_client(
    client: &reqwest::blocking::Client,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, DownloadError> {
    fs::create_dir_all(output_dir)?;
    let code_dir = output_dir.join(super::gaze::HISTORY_DRAW_CODE);
    fs::create_dir_all(&code_dir)?;

    let mut saved_files = Vec::new();
    for year in FALLBACK_START_YEAR..=FALLBACK_MAX_YEAR {
        let metadata = match resolve_history_zip_for_year(client, year)? {
            Some(value) => value,
            None => break,
        };

        let download_path = metadata
            .path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                std::io::Error::other("Taiwan Lottery API returned empty download path")
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
        return Err(std::io::Error::other(
            "no downloadable history draw zip in Taiwan Lottery API",
        )
        .into());
    }

    Ok(saved_files)
}

pub fn download_history_draw(output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    download_history_draw_with_client(&client, output_dir)
}
