use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::DownloadError;

use super::common::{
    build_http_client, extract_zip_bytes, pick_download_file_name, should_extract_zip,
    zip_extract_dir_for_file,
};

const CSV_BASE_URL: &str = "https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=";
const API_DOCS_URL: &str = "https://gaze.nta.gov.tw/ntaOpenApi/v2/api-docs?group=FinancialPlanning";
const API_DOCS_FILE_NAME: &str = "financialplanning_api_docs.json";
pub(crate) const HISTORY_DRAW_CODE: &str = "D423F";

#[derive(Debug, serde::Deserialize)]
struct ApiDocs {
    paths: HashMap<String, serde_json::Value>,
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

pub(crate) fn parse_download_links_from_csv(
    csv_bytes: &[u8],
) -> Result<Vec<String>, DownloadError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(csv_bytes);

    let mut links = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for record in reader.records() {
        let record = record?;
        for field in &record {
            let candidate = field.trim();
            if (candidate.starts_with("http://") || candidate.starts_with("https://"))
                && seen.insert(candidate.to_string())
            {
                links.push(candidate.to_string());
            }
        }
    }

    Ok(links)
}

fn download_csv_linked_files(
    client: &reqwest::blocking::Client,
    code: &str,
    csv_bytes: &[u8],
    output_dir: &Path,
) -> Result<Vec<PathBuf>, DownloadError> {
    let links = parse_download_links_from_csv(csv_bytes)?;
    if links.is_empty() {
        return Ok(Vec::new());
    }

    let code_dir = output_dir.join(code);
    fs::create_dir_all(&code_dir)?;

    let mut used_file_names = std::collections::HashSet::new();
    let mut saved_files = Vec::with_capacity(links.len());
    for (index, link) in links.iter().enumerate() {
        let response = client.get(link).send()?.error_for_status()?;
        let headers = response.headers().clone();
        let mut file_name = pick_download_file_name(link, &headers, index + 1);
        let file_bytes = response.bytes()?;

        if file_name.rsplit_once('.').is_none() {
            if let Some(extension) = super::common::extension_from_magic_bytes(&file_bytes) {
                file_name.push('.');
                file_name.push_str(extension);
            }
        }

        if !used_file_names.insert(file_name.clone()) {
            file_name = format!("{}_{}", index + 1, file_name);
            used_file_names.insert(file_name.clone());
        }

        let out_path = code_dir.join(file_name);
        fs::write(&out_path, &file_bytes)?;
        saved_files.push(out_path.clone());

        let saved_name = out_path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if should_extract_zip(saved_name, &file_bytes) {
            let extract_dir = zip_extract_dir_for_file(&code_dir, saved_name, index + 1);
            let extracted_files = extract_zip_bytes(&file_bytes, &extract_dir)?;
            saved_files.extend(extracted_files);
        }
    }

    Ok(saved_files)
}

fn download_api_doc_with_client(
    client: &reqwest::blocking::Client,
    output_dir: &Path,
) -> Result<(String, PathBuf), DownloadError> {
    fs::create_dir_all(output_dir)?;

    let api_docs_body = client
        .get(API_DOCS_URL)
        .send()?
        .error_for_status()?
        .text()?;

    let api_docs_out_path = output_dir.join(API_DOCS_FILE_NAME);
    fs::write(&api_docs_out_path, api_docs_body.as_bytes())?;
    Ok((api_docs_body, api_docs_out_path))
}

fn download_dataset_with_client(
    client: &reqwest::blocking::Client,
    output_dir: &Path,
    code: &str,
) -> Result<Vec<PathBuf>, DownloadError> {
    fs::create_dir_all(output_dir)?;

    let url = build_csv_url(code);
    let body = client.get(&url).send()?.error_for_status()?.bytes()?;

    let out_path = output_dir.join(format!("{code}.csv"));
    fs::write(&out_path, &body)?;

    let mut saved_files = Vec::new();
    saved_files.push(out_path);

    let linked_files = download_csv_linked_files(client, code, &body, output_dir)?;
    saved_files.extend(linked_files);
    Ok(saved_files)
}

pub fn download_api_doc(output_dir: impl AsRef<Path>) -> Result<PathBuf, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    let (_, path) = download_api_doc_with_client(&client, output_dir)?;
    Ok(path)
}

pub fn download_dataset(
    output_dir: impl AsRef<Path>,
    dataset_code: &str,
) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    download_dataset_with_client(&client, output_dir, dataset_code)
}

pub fn download_history_draw(output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    download_dataset_with_client(&client, output_dir, HISTORY_DRAW_CODE)
}

pub fn download_all(output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    let (api_docs_body, api_docs_out_path) = download_api_doc_with_client(&client, output_dir)?;

    let codes = parse_codes_from_api_docs(&api_docs_body)?;

    let mut saved_files = Vec::with_capacity(codes.len() + 1);
    saved_files.push(api_docs_out_path);
    for code in codes {
        let files = download_dataset_with_client(&client, output_dir, &code)?;
        saved_files.extend(files);
    }

    Ok(saved_files)
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

    #[test]
    fn history_draw_targets_d423f() {
        assert_eq!(build_csv_url(HISTORY_DRAW_CODE), build_csv_url("D423F"));
    }

    #[test]
    fn parse_download_links_from_csv_works() {
        let sample = "年度,下載連結\n109, https://example.com/a \n110,https://example.com/b\n111,https://example.com/a\n";

        let links = parse_download_links_from_csv(sample.as_bytes()).expect("must parse links");
        assert_eq!(
            links,
            vec![
                "https://example.com/a".to_string(),
                "https://example.com/b".to_string(),
            ]
        );
    }

    #[test]
    fn content_disposition_filename_star_utf8_is_decoded() {
        let header = "attachment; filename*=UTF-8''%E6%B8%AC%E8%A9%A6%E5%A0%B1%E8%A1%A8.pdf";
        let file_name = crate::download::common::file_name_from_content_disposition(header).expect("must parse filename");
        assert_eq!(file_name, "測試報表.pdf");
    }

    #[test]
    fn content_disposition_filename_percent_encoded_is_decoded() {
        let header = "attachment; filename=%E6%B8%AC%E8%A9%A6%E8%B3%87%E6%96%99.xlsx";
        let file_name = crate::download::common::file_name_from_content_disposition(header).expect("must parse filename");
        assert_eq!(file_name, "測試資料.xlsx");
    }

    #[test]
    fn extension_from_content_type_is_appended_when_missing() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/pdf".parse().expect("valid content type"),
        );

        let file_name = pick_download_file_name(
            "https://www.nta.gov.tw/download/08a270e17516429092f32b3bcbae78cb",
            &headers,
            1,
        );
        assert_eq!(file_name, "08a270e17516429092f32b3bcbae78cb.pdf");
    }
}
