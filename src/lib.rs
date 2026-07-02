use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use encoding_rs::BIG5;
use reqwest::header::{CONTENT_DISPOSITION, CONTENT_TYPE};

mod errors;
mod ffi;

pub use errors::DownloadError;

const CSV_BASE_URL: &str = "https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=";
const API_DOCS_URL: &str = "https://gaze.nta.gov.tw/ntaOpenApi/v2/api-docs?group=FinancialPlanning";
const API_DOCS_FILE_NAME: &str = "financialplanning_api_docs.json";
const HISTORY_DRAW_CODE: &str = "D423F";
const TAIWAN_LOTTERY_API_BASE_URL: &str = "https://api.taiwanlottery.com/TLCAPIWeB";
const TAIWAN_LOTTERY_RESULT_DOWNLOAD_URL: &str = "https://api.taiwanlottery.com/TLCAPIWeB/Lottery/ResultDownload";
const TAIWAN_LOTTERY_FALLBACK_START_YEAR: i32 = 2007;
const TAIWAN_LOTTERY_FALLBACK_MAX_YEAR: i32 = 2200;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistorySession {
    Third,
    Fourth,
    Fifth,
}

impl Default for HistorySession {
    fn default() -> Self {
        Self::Fifth
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoryGame {
    SuperLotto638,
    Lotto649,
    Daily539,
    Lotto3D,
    Lotto4D,
    Lotto49M6,
    Lotto39M5,
    Lotto38M6,
    Lotto1224,
    Lotto740,
    TicTacToe,
    Lotto638,
}

impl HistoryGame {
    fn path(self) -> &'static str {
        match self {
            Self::SuperLotto638 => "/Lottery/SuperLotto638Result",
            Self::Lotto649 => "/Lottery/Lotto649Result",
            Self::Daily539 => "/Lottery/Daily539Result",
            Self::Lotto3D => "/Lottery/3DResult",
            Self::Lotto4D => "/Lottery/4DResult",
            Self::Lotto49M6 => "/Lottery/49M6Result",
            Self::Lotto39M5 => "/Lottery/39M5Result",
            Self::Lotto38M6 => "/Lottery/38M6Result",
            Self::Lotto1224 => "/Lottery/Lotto1224Result",
            Self::Lotto740 => "/Lottery/Lotto740Result",
            Self::TicTacToe => "/Lottery/TicTacToeResult",
            Self::Lotto638 => "/Lottery/Lotto638Result",
        }
    }

    fn history_session_path(self) -> Option<&'static str> {
        match self {
            Self::Lotto3D => Some("/Lottery/3DHistoryResult"),
            Self::Lotto4D => Some("/Lottery/4DHistoryResult"),
            _ => None,
        }
    }

    fn api_path_for_session(self, session: HistorySession) -> &'static str {
        if session != HistorySession::Fifth {
            if let Some(history_path) = self.history_session_path() {
                return history_path;
            }
        }

        self.path()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryDrawQuery {
    pub period: Option<String>,
    pub month: Option<String>,
    pub end_month: Option<String>,
    pub session: HistorySession,
}

impl Default for HistoryDrawQuery {
    fn default() -> Self {
        Self {
            period: None,
            month: None,
            end_month: None,
            session: HistorySession::Fifth,
        }
    }
}

impl HistoryDrawQuery {
    pub fn by_period(period: impl Into<String>) -> Self {
        Self {
            period: Some(period.into()),
            ..Self::default()
        }
    }

    pub fn by_month(month: impl Into<String>) -> Self {
        let month = month.into();
        Self {
            month: Some(month.clone()),
            end_month: Some(month),
            ..Self::default()
        }
    }

    pub fn by_month_range(month: impl Into<String>, end_month: impl Into<String>) -> Self {
        Self {
            month: Some(month.into()),
            end_month: Some(end_month.into()),
            ..Self::default()
        }
    }

    fn normalized_params(&self) -> Result<(&str, &str, &str), DownloadError> {
        let period = self.period.as_deref().unwrap_or("").trim();
        if !period.is_empty() {
            return Ok((period, "", ""));
        }

        let month = self
            .month
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                std::io::Error::other("month is required when period is empty")
            })?;
        let end_month = self
            .end_month
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(month);

        Ok(("", month, end_month))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HistoryDrawItem {
    pub period: String,
    pub lottery_date: Option<String>,
    pub redeemable_date: Option<String>,
    pub draw_number_size: Vec<i32>,
    pub draw_number_appear: Option<Vec<i32>>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HistoryDrawPage {
    pub total_size: usize,
    pub items: Vec<HistoryDrawItem>,
}

#[derive(Debug, Clone)]
struct LocalHistoryDrawRecord {
    period: String,
    lottery_date: Option<String>,
    draw_number_appear: Vec<i32>,
}

#[derive(Debug, serde::Deserialize)]
struct ApiDocs {
    paths: HashMap<String, serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
struct TaiwanLotteryResultDownloadResponse {
    #[serde(rename = "rtCode")]
    rt_code: i32,
    content: Option<TaiwanLotteryResultDownloadContent>,
}

#[derive(Debug, serde::Deserialize)]
struct TaiwanLotteryResultDownloadContent {
    #[serde(rename = "fileName")]
    file_name: Option<String>,
    path: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TaiwanLotteryHistoryResponse {
    #[serde(rename = "rtCode")]
    rt_code: i32,
    #[serde(rename = "rtMsg")]
    rt_msg: Option<String>,
    content: Option<serde_json::Value>,
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

fn parse_download_links_from_csv(csv_bytes: &[u8]) -> Result<Vec<String>, DownloadError> {
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

fn sanitize_file_name(file_name: &str) -> String {
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

    if sanitized.is_empty() {
        "download.bin".to_string()
    } else {
        sanitized
    }
}

fn file_name_from_content_disposition(header_value: &str) -> Option<String> {
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

fn header_value_to_text(value: &reqwest::header::HeaderValue) -> String {
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

fn extension_from_magic_bytes(file_bytes: &[u8]) -> Option<&'static str> {
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

fn should_extract_zip(file_name: &str, file_bytes: &[u8]) -> bool {
    file_name.to_ascii_lowercase().ends_with(".zip")
        || matches!(extension_from_magic_bytes(file_bytes), Some("zip"))
}

fn zip_extract_dir_for_file(code_dir: &Path, file_name: &str, fallback_index: usize) -> PathBuf {
    let stem = Path::new(file_name)
        .file_stem()
        .and_then(|value| value.to_str())
        .map(sanitize_file_name)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("archive_{fallback_index}"));

    code_dir.join(stem)
}

fn decode_non_utf8_zip_name(raw_name: &[u8], fallback_name: &str) -> String {
    let (decoded, _, had_errors) = BIG5.decode(raw_name);
    if had_errors {
        fallback_name.to_string()
    } else {
        decoded.into_owned()
    }
}

fn normalize_zip_entry_path(entry_name: &str) -> Option<PathBuf> {
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

fn extract_zip_bytes(zip_bytes: &[u8], extract_dir: &Path) -> Result<Vec<PathBuf>, DownloadError> {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(zip_bytes))?;
    fs::create_dir_all(extract_dir)?;

    let mut extracted_files = Vec::new();
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

        let mut out_file = fs::File::create(&out_path)?;
        std::io::copy(&mut entry, &mut out_file)?;
        extracted_files.push(out_path);
    }

    Ok(extracted_files)
}

fn pick_download_file_name(
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
        link
        .split('?')
        .next()
        .and_then(|path| path.rsplit('/').next())
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(sanitize_file_name)
    })
    .unwrap_or_else(|| format!("download_{fallback_index}.bin"));

    if file_name.rsplit_once('.').is_none()
        && let Some(extension) = extension_from_content_type(headers)
    {
        file_name.push('.');
        file_name.push_str(extension);
    }

    file_name
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

        if file_name.rsplit_once('.').is_none()
            && let Some(extension) = extension_from_magic_bytes(&file_bytes)
        {
            file_name.push('.');
            file_name.push_str(extension);
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

fn build_http_client() -> Result<reqwest::blocking::Client, DownloadError> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(DownloadError::from)
}

fn json_value_to_i32_vec(value: Option<&serde_json::Value>) -> Vec<i32> {
    let Some(serde_json::Value::Array(values)) = value else {
        return Vec::new();
    };

    values
        .iter()
        .filter_map(|entry| entry.as_i64())
        .filter_map(|entry| i32::try_from(entry).ok())
        .collect()
}

fn parse_history_draw_page(content: &serde_json::Value) -> Result<HistoryDrawPage, DownloadError> {
    let serde_json::Value::Object(content_obj) = content else {
        return Err(std::io::Error::other("history response content is not an object").into());
    };

    let total_size = content_obj
        .get("totalSize")
        .and_then(|value| value.as_u64())
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0);

    let records = content_obj
        .values()
        .find_map(|value| {
            let serde_json::Value::Array(records) = value else {
                return None;
            };

            let has_draw_fields = records.iter().any(|record| {
                let serde_json::Value::Object(record_obj) = record else {
                    return false;
                };

                record_obj.contains_key("drawNumberAppear") || record_obj.contains_key("drawNumberSize")
            });

            if has_draw_fields {
                Some(records)
            } else {
                None
            }
        })
        .ok_or_else(|| {
            std::io::Error::other("history response does not include draw records")
        })?;

    let mut items = Vec::new();
    for record in records {
        let serde_json::Value::Object(record_obj) = record else {
            continue;
        };

        let draw_number_size = json_value_to_i32_vec(record_obj.get("drawNumberSize"));
        let draw_number_appear = json_value_to_i32_vec(record_obj.get("drawNumberAppear"));
        if draw_number_size.is_empty() && draw_number_appear.is_empty() {
            continue;
        }

        let period = match record_obj.get("period") {
            Some(serde_json::Value::String(value)) => value.clone(),
            Some(serde_json::Value::Number(value)) => value.to_string(),
            _ => String::new(),
        };

        let lottery_date = record_obj
            .get("lotteryDate")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned);
        let redeemable_date = record_obj
            .get("redeemableDate")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned);

        items.push(HistoryDrawItem {
            period,
            lottery_date,
            redeemable_date,
            draw_number_size,
            draw_number_appear: Some(draw_number_appear),
        });
    }

    Ok(HistoryDrawPage { total_size, items })
}

fn get_history_draw_with_client(
    client: &reqwest::blocking::Client,
    game: HistoryGame,
    query: &HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    let (period, month, end_month) = query.normalized_params()?;
    let url = format!("{TAIWAN_LOTTERY_API_BASE_URL}{}", game.api_path_for_session(query.session));

    let page_size = 200usize;
    let mut page_num = 1usize;
    let mut total_size = 0usize;
    let mut all_items = Vec::new();

    loop {
        let response_body = client
            .get(&url)
            .query(&[
                ("period", period),
                ("month", month),
                ("endMonth", end_month),
                ("pageNum", &page_num.to_string()),
                ("pageSize", &page_size.to_string()),
            ])
            .send()?
            .error_for_status()?
            .text()?;

        let response: TaiwanLotteryHistoryResponse = serde_json::from_str(&response_body)?;
        if response.rt_code != 0 {
            let message = response
                .rt_msg
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unknown history API error");
            return Err(std::io::Error::other(format!("Taiwan Lottery history API returned rtCode={}, msg={message}", response.rt_code)).into());
        }

        let content = response
            .content
            .as_ref()
            .ok_or_else(|| std::io::Error::other("Taiwan Lottery history API returned empty content"))?;
        let page = parse_history_draw_page(content)?;

        if page_num == 1 {
            total_size = page.total_size;
        }

        if page.items.is_empty() {
            break;
        }

        let fetched = page.items.len();
        all_items.extend(page.items);

        if fetched < page_size {
            break;
        }
        if total_size > 0 && all_items.len() >= total_size {
            break;
        }

        page_num += 1;
    }

    Ok(HistoryDrawPage {
        total_size: total_size.max(all_items.len()),
        items: all_items,
    })
}

fn history_game_file_prefixes(game: HistoryGame) -> &'static [&'static str] {
    match game {
        HistoryGame::SuperLotto638 => &["威力彩_"],
        HistoryGame::Lotto649 => &["大樂透_"],
        HistoryGame::Daily539 => &["今彩539_"],
        HistoryGame::Lotto3D => &["3星彩_", "三星彩_"],
        HistoryGame::Lotto4D => &["4星彩_", "四星彩_"],
        HistoryGame::Lotto49M6 => &["49樂合彩_"],
        HistoryGame::Lotto39M5 => &["39樂合彩_"],
        HistoryGame::Lotto38M6 => &["38樂合彩_", "大樂透加開獎項_"],
        HistoryGame::Lotto1224 => &["BINGOBINGO賓果賓果_", "賓果賓果_"],
        HistoryGame::Lotto740 => &["大福彩_", "Lotto740_"],
        HistoryGame::TicTacToe => &["tictactoe_", "賓果賓果_"],
        HistoryGame::Lotto638 => &["賓果賓果_", "BINGOBINGO賓果賓果_"],
    }
}

fn resolve_history_data_root(output_dir: &Path) -> Result<PathBuf, DownloadError> {
    if output_dir
        .file_name()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case(HISTORY_DRAW_CODE))
    {
        return Ok(output_dir.to_path_buf());
    }

    let d423f_dir = output_dir.join(HISTORY_DRAW_CODE);
    if d423f_dir.exists() {
        Ok(d423f_dir)
    } else {
        Err(std::io::Error::other(format!(
            "history data directory not found: {}",
            d423f_dir.display()
        ))
        .into())
    }
}

fn collect_history_csv_files(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), DownloadError> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            collect_history_csv_files(&path, output)?;
            continue;
        }

        let is_csv = path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("csv"));
        if is_csv {
            output.push(path);
        }
    }

    Ok(())
}

fn parse_lottery_date_month(lottery_date: &str) -> Option<String> {
    let normalized = lottery_date.trim().replace('/', "-");
    if normalized.len() >= 7 {
        Some(normalized[..7].to_string())
    } else {
        None
    }
}

fn extract_draw_numbers(headers: &csv::StringRecord, record: &csv::StringRecord) -> Vec<i32> {
    headers
        .iter()
        .enumerate()
        .filter(|(_, header)| {
            let header = header.trim();
            header.starts_with("獎號")
                || header == "特別號"
                || header == "第二區"
                || header == "第二區號"
        })
        .filter_map(|(index, _)| record.get(index))
        .filter_map(|value| value.trim().parse::<i32>().ok())
        .collect()
}

fn parse_history_csv_file(file_path: &Path) -> Result<Vec<LocalHistoryDrawRecord>, DownloadError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_path(file_path)?;
    let headers = reader.headers()?.clone();

    let period_index = headers
        .iter()
        .position(|header| header.trim() == "期別")
        .ok_or_else(|| {
            std::io::Error::other(format!(
                "history csv missing period column: {}",
                file_path.display()
            ))
        })?;
    let lottery_date_index = headers.iter().position(|header| header.trim() == "開獎日期");

    let mut records = Vec::new();
    for row in reader.records() {
        let row = row?;
        let period = row.get(period_index).unwrap_or_default().trim().to_string();
        if period.is_empty() {
            continue;
        }

        let draw_number_appear = extract_draw_numbers(&headers, &row);
        if draw_number_appear.is_empty() {
            continue;
        }

        let lottery_date = lottery_date_index
            .and_then(|index| row.get(index))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        records.push(LocalHistoryDrawRecord {
            period,
            lottery_date,
            draw_number_appear,
        });
    }

    Ok(records)
}

fn get_history_draw_from_downloaded_data(
    output_dir: &Path,
    game: HistoryGame,
    query: &HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    let (period, month, _) = query.normalized_params()?;
    let root = resolve_history_data_root(output_dir)?;

    let prefixes = history_game_file_prefixes(game);
    let mut csv_files = Vec::new();
    collect_history_csv_files(&root, &mut csv_files)?;
    csv_files.retain(|path| {
        path.file_name()
            .and_then(|value| value.to_str())
            .is_some_and(|name| prefixes.iter().any(|prefix| name.starts_with(prefix)))
    });

    let mut all_records = Vec::new();
    for file_path in csv_files {
        let mut file_records = parse_history_csv_file(&file_path)?;
        all_records.append(&mut file_records);
    }

    if !period.is_empty() {
        all_records.retain(|record| record.period == period);
    } else {
        all_records.retain(|record| {
            record
                .lottery_date
                .as_deref()
                .and_then(parse_lottery_date_month)
                .is_some_and(|value| value == month)
        });
    }

    all_records.sort_by(|left, right| right.period.cmp(&left.period));
    all_records.dedup_by(|left, right| left.period == right.period);

    let total_size = all_records.len();
    let items = all_records
        .iter()
        .map(|record| {
            let draw_number_size = record.draw_number_appear.clone();

            HistoryDrawItem {
                period: record.period.clone(),
                lottery_date: record.lottery_date.clone(),
                redeemable_date: None,
                draw_number_size,
                draw_number_appear: None,
            }
        })
        .collect();

    Ok(HistoryDrawPage { total_size, items })
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

fn resolve_taiwan_lottery_history_zip_for_year(
    client: &reqwest::blocking::Client,
    year: i32,
) -> Result<Option<TaiwanLotteryResultDownloadContent>, DownloadError> {
    let response = client
        .get(TAIWAN_LOTTERY_RESULT_DOWNLOAD_URL)
        .query(&[("year", year)])
        .send()?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let response_body = response.text()?;
    let parsed: TaiwanLotteryResultDownloadResponse = serde_json::from_str(&response_body)?;
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

fn download_history_draw_from_taiwan_lottery_with_client(
    client: &reqwest::blocking::Client,
    output_dir: &Path,
) -> Result<Vec<PathBuf>, DownloadError> {
    fs::create_dir_all(output_dir)?;
    let code_dir = output_dir.join(HISTORY_DRAW_CODE);
    fs::create_dir_all(&code_dir)?;

    let mut saved_files = Vec::new();
    for year in TAIWAN_LOTTERY_FALLBACK_START_YEAR..=TAIWAN_LOTTERY_FALLBACK_MAX_YEAR {
        let metadata = match resolve_taiwan_lottery_history_zip_for_year(client, year)? {
            Some(value) => value,
            None => break,
        };

        let download_path = metadata
            .path
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| std::io::Error::other("Taiwan Lottery API returned empty download path"))?;

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
        return Err(std::io::Error::other("no downloadable history draw zip in Taiwan Lottery API").into());
    }

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

pub fn download_history_draw_from_gov_data(
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    download_dataset_with_client(&client, output_dir, HISTORY_DRAW_CODE)
}

pub fn download_history_draw_from_taiwan_lottery(
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    download_history_draw_from_taiwan_lottery_with_client(&client, output_dir)
}

pub fn download_history_draw(output_dir: impl AsRef<Path>) -> Result<Vec<PathBuf>, DownloadError> {
    match download_history_draw_from_gov_data(output_dir.as_ref()) {
        Ok(files) => Ok(files),
        Err(DownloadError::Http(_)) => download_history_draw_from_taiwan_lottery(output_dir.as_ref()),
        Err(err) => Err(err),
    }
}

pub fn get_history_draw(
    output_dir: impl AsRef<Path>,
    game: HistoryGame,
    query: HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    get_history_draw_from_downloaded_data(output_dir.as_ref(), game, &query)
}

pub fn get_history_draw_from_taiwan_lottory(
    game: HistoryGame,
    query: HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    let client = build_http_client()?;
    get_history_draw_with_client(&client, game, &query)
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
        let file_name = file_name_from_content_disposition(header).expect("must parse filename");
        assert_eq!(file_name, "測試報表.pdf");
    }

    #[test]
    fn content_disposition_filename_percent_encoded_is_decoded() {
        let header = "attachment; filename=%E6%B8%AC%E8%A9%A6%E8%B3%87%E6%96%99.xlsx";
        let file_name = file_name_from_content_disposition(header).expect("must parse filename");
        assert_eq!(file_name, "測試資料.xlsx");
    }

    #[test]
    fn mojibake_utf8_filename_is_fixed() {
        let mojibake: String = [
            0xE4u8, 0xB8, 0xAD, 0xE6, 0x96, 0x87, 0xE5, 0xA0, 0xB1, 0xE8, 0xA1, 0xA8, b'.',
            b'p', b'd', b'f',
        ]
        .iter()
        .map(|byte| *byte as char)
        .collect();
        let header = format!("attachment; filename=\"{mojibake}\"");
        let file_name = file_name_from_content_disposition(&header).expect("must parse filename");
        assert_eq!(file_name, "中文報表.pdf");
    }

    #[test]
    fn extension_from_content_type_is_appended_when_missing() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(CONTENT_TYPE, "application/pdf".parse().expect("valid content type"));

        let file_name = pick_download_file_name(
            "https://www.nta.gov.tw/download/08a270e17516429092f32b3bcbae78cb",
            &headers,
            1,
        );
        assert_eq!(file_name, "08a270e17516429092f32b3bcbae78cb.pdf");
    }

    #[test]
    fn extension_from_magic_bytes_detects_pdf() {
        let bytes = b"%PDF-1.7\r\n...";
        assert_eq!(extension_from_magic_bytes(bytes), Some("pdf"));
    }

    #[test]
    fn raw_header_bytes_filename_is_decoded() {
        let raw = b"form-data; name=\"attachment\"; filename=\"112\xE5\xB9\xB4\xE5\xBA\xA6\xE5\x85\xAC\xE7\x9B\x8A\xE5\xBD\xA9\xE5\x88\xB8.pdf\"";
        let value = reqwest::header::HeaderValue::from_bytes(raw).expect("valid header bytes");
        let text = header_value_to_text(&value);
        let file_name = file_name_from_content_disposition(&text).expect("must decode filename");
        assert_eq!(file_name, "112年度公益彩券.pdf");
    }

    #[test]
    fn should_extract_zip_matches_extension_or_magic_bytes() {
        assert!(should_extract_zip("a.zip", b"not-zip"));
        assert!(should_extract_zip("a.bin", &[0x50, 0x4B, 0x03, 0x04]));
        assert!(!should_extract_zip("a.pdf", b"%PDF-1.7"));
    }

    #[test]
    fn extract_zip_bytes_unpacks_files() {
        use std::io::Write as _;

        let cursor = std::io::Cursor::new(Vec::<u8>::new());
        let mut writer = zip::ZipWriter::new(cursor);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        writer
            .start_file("nested/hello.txt", options)
            .expect("start file in zip");
        writer.write_all(b"hello zip").expect("write zip content");
        let zip_bytes = writer.finish().expect("finish zip").into_inner();

        let extract_dir = std::env::temp_dir().join(format!(
            "taiwan-lottery-zip-test-{}",
            std::process::id()
        ));
        if extract_dir.exists() {
            fs::remove_dir_all(&extract_dir).expect("remove old test directory");
        }

        let extracted = extract_zip_bytes(&zip_bytes, &extract_dir).expect("extract zip files");
        assert_eq!(extracted.len(), 1);

        let extracted_file = extract_dir.join("nested").join("hello.txt");
        assert!(extracted_file.exists());
        let content = fs::read_to_string(&extracted_file).expect("read extracted file");
        assert_eq!(content, "hello zip");

        fs::remove_dir_all(&extract_dir).expect("cleanup extracted test files");
    }

    #[test]
    fn decode_non_utf8_zip_name_supports_big5() {
        let (encoded, _, had_errors) = BIG5.encode("中文檔名.csv");
        assert!(!had_errors);

        let decoded = decode_non_utf8_zip_name(encoded.as_ref(), "fallback.csv");
        assert_eq!(decoded, "中文檔名.csv");
    }

    #[test]
    fn normalize_zip_entry_path_rejects_parent_traversal_components() {
        let path = normalize_zip_entry_path("../unsafe/..//ok.csv").expect("normalized path");
        assert_eq!(path, PathBuf::from("unsafe").join("ok.csv"));
    }

    #[test]
    fn history_game_3d_uses_history_endpoint_outside_fifth_session() {
        assert_eq!(
            HistoryGame::Lotto3D.api_path_for_session(HistorySession::Third),
            "/Lottery/3DHistoryResult"
        );
        assert_eq!(
            HistoryGame::Lotto3D.api_path_for_session(HistorySession::Fifth),
            "/Lottery/3DResult"
        );
    }

    #[test]
    fn parse_history_draw_page_extracts_both_draw_orders() {
        let sample = serde_json::json!({
            "totalSize": 1,
            "lotto649Res": [
                {
                    "period": 112000116,
                    "lotteryDate": "2023-12-29T00:00:00",
                    "redeemableDate": "2024-04-01T00:00:00",
                    "drawNumberSize": [1, 11, 23, 31, 39, 46, 17],
                    "drawNumberAppear": [31, 46, 11, 39, 23, 1, 17]
                }
            ]
        });

        let page = parse_history_draw_page(&sample).expect("parse history draw page");
        assert_eq!(page.total_size, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].period, "112000116");
        assert_eq!(page.items[0].draw_number_size, vec![1, 11, 23, 31, 39, 46, 17]);
        assert_eq!(
            page.items[0].draw_number_appear,
            Some(vec![31, 46, 11, 39, 23, 1, 17])
        );
    }

    #[test]
    fn history_draw_query_requires_month_when_period_is_empty() {
        let query = HistoryDrawQuery::default();
        let err = query.normalized_params().expect_err("must fail without period or month");
        assert!(matches!(err, DownloadError::Io(_)));
    }

    #[test]
    fn get_history_draw_reads_downloaded_csv_data() {
        let root = std::env::temp_dir().join(format!(
            "taiwan-lottery-history-local-test-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("cleanup old temp dir");
        }

        let game_dir = root.join("D423F").join("2026");
        fs::create_dir_all(&game_dir).expect("create game dir");
        let file = game_dir.join("大樂透_2026.csv");
        fs::write(
            &file,
            "遊戲名稱,期別,開獎日期,獎號1,獎號2,獎號3,獎號4,獎號5,獎號6,特別號\n大樂透,115000001,2026/01/02,3,7,16,19,40,42,12\n",
        )
        .expect("write csv");

        let query = HistoryDrawQuery::by_period("115000001");
        let page = get_history_draw(&root, HistoryGame::Lotto649, query).expect("query local data");
        assert_eq!(page.total_size, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].draw_number_appear, None);
        assert_eq!(page.items[0].draw_number_size, vec![3, 7, 16, 19, 40, 42, 12]);

        fs::remove_dir_all(&root).expect("cleanup temp dir");
    }
}
