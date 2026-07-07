//! Taiwan Lottery library providing download, query, and random draw capabilities.
//!
//! This library offers both Rust and C APIs for:
//! - **Download**: Fetch lottery datasets from Taiwan's open data sources
//! - **Query**: Search historical lottery results by period, month, or month range
//! - **Draw**: Generate random lottery draws for supported games
//!
//! # Overview
//!
//! The library uses two primary data sources:
//! - **FinancialPlanning OpenData** (via `D423F` dataset) - Primary source for historical draws
//! - **Taiwan Lottery API** - Fallback source for recent results
//!
//! # Quick Start
//!
//! ## Download historical data
//!
//! ```ignore
//! use taiwan_lottery::download_history_draw;
//!
//! download_history_draw("./data")?;
//! ```
//!
//! ## Query results
//!
//! ```ignore
//! use taiwan_lottery::{query_history_draw, HistoryDrawQuery, LotteryGame};
//!
//! let query = HistoryDrawQuery::by_month("2023-12");
//! let results = query_history_draw("./data", LotteryGame::Lotto649, query)?;
//! ```
//!
//! ## Generate random draw
//!
//! ```ignore
//! use taiwan_lottery::{draw_by_game, LotteryGame};
//!
//! let result = draw_by_game(LotteryGame::Lotto649);
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use encoding_rs::BIG5;
use reqwest::header::{CONTENT_DISPOSITION, CONTENT_TYPE};

mod download;
mod draw;
mod errors;
mod ffi;
mod numbers;
mod query;
mod rule;

use query::remote::game_query_month_bounds;
use rule::metadata_for_game;

pub use download::{
    download_all, download_api_doc, download_dataset, download_history_draw,
    download_history_draw_from_gov_data, download_history_draw_from_taiwan_lottery,
};
pub use draw::{draw_by_game, DrawResult};
pub use errors::DownloadError;
pub use numbers::{
    BingoBingoNumbers, BonusDrawNumbers, Daily539Numbers, DrawNumbers, Lotto1224Numbers,
    Lotto38M6Numbers, Lotto39M5Numbers, Lotto3DNumbers, Lotto49M6Numbers, Lotto4DNumbers,
    Lotto638Numbers, Lotto649Numbers, Lotto740Numbers, SortedDrawNumbers, SuperLotto638Numbers,
    TicTacToeNumbers,
};
pub use query::{query_history_draw, query_history_draw_from_taiwan_lottery};

const CSV_BASE_URL: &str = "https://gaze.nta.gov.tw/dntmb/OpenData/csvDw?ntaCode=";
const API_DOCS_URL: &str = "https://gaze.nta.gov.tw/ntaOpenApi/v2/api-docs?group=FinancialPlanning";
const API_DOCS_FILE_NAME: &str = "financialplanning_api_docs.json";
const HISTORY_DRAW_CODE: &str = "D423F";
const TAIWAN_LOTTERY_RESULT_DOWNLOAD_URL: &str =
    "https://api.taiwanlottery.com/TLCAPIWeB/Lottery/ResultDownload";
const TAIWAN_LOTTERY_FALLBACK_START_YEAR: i32 = 2007;
const TAIWAN_LOTTERY_FALLBACK_MAX_YEAR: i32 = 2200;

/// Supported lottery games for historical draw queries and random draws.
///
/// Each variant represents a different Taiwan lottery game. Use this with
/// [`query_history_draw`], [`query_history_draw_from_taiwan_lottery`], or [`draw_by_game`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LotteryGame {
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
    BingoBingo,
}

/// One number selection segment for a lottery game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LotteryGameNumberRule {
    /// Segment name such as `main`, `bonus`, `super`, or `zone_1`.
    pub name: &'static str,
    /// How many numbers are selected from this segment.
    pub picks: usize,
    /// Inclusive minimum value for this segment.
    pub min: i32,
    /// Inclusive maximum value for this segment.
    pub max: i32,
    /// Whether values in this segment may repeat.
    pub allow_repeat: bool,
}

/// Static metadata for rendering lottery game information in UI layers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LotteryGameMetadata {
    /// UI display name for the game. Defaults to English.
    pub display_name: &'static str,
    /// English display name for the game.
    pub display_name_english: &'static str,
    /// Chinese display name for the game.
    pub display_name_chinese: &'static str,
    /// Human-readable summary of the game's number-selection rule.
    pub number_rule: &'static str,
    /// Rule segments that together define how numbers are selected.
    pub number_ranges: &'static [LotteryGameNumberRule],
}

/// Remote query parameter support for one lottery game endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RemoteQueryParamSupport {
    pub month: bool,
    pub end_month: bool,
    pub open_date: bool,
    pub period: bool,
}

/// Supported display-name languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LotteryDisplayLanguage {
    #[default]
    English,
    Chinese,
}

impl LotteryGameMetadata {
    /// Resolves a display name by enum language.
    pub const fn display_name_for_language(self, language: LotteryDisplayLanguage) -> &'static str {
        match language {
            LotteryDisplayLanguage::English => self.display_name_english,
            LotteryDisplayLanguage::Chinese => self.display_name_chinese,
        }
    }

    /// Returns metadata with `display_name` localized by input language.
    pub fn with_display_language(mut self, language: LotteryDisplayLanguage) -> Self {
        self.display_name = self.display_name_for_language(language);
        self
    }
}

/// Queryable month range for a game in `YYYY-MM` format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotteryGameQueryRange {
    /// Earliest supported query month.
    pub min_month: String,
    /// Latest supported query month.
    pub max_month: String,
}

#[deprecated(note = "use LotteryGame instead")]
pub type HistoryGame = LotteryGame;

#[deprecated(note = "use LotteryGameNumberRule instead")]
pub type HistoryGameNumberRule = LotteryGameNumberRule;

#[deprecated(note = "use LotteryGameMetadata instead")]
pub type HistoryGameMetadata = LotteryGameMetadata;

impl LotteryGame {
    /// All supported games in a stable order suitable for UI lists.
    pub const ALL: [Self; 13] = [
        Self::SuperLotto638,
        Self::Lotto649,
        Self::Daily539,
        Self::Lotto3D,
        Self::Lotto4D,
        Self::Lotto49M6,
        Self::Lotto39M5,
        Self::Lotto38M6,
        Self::Lotto1224,
        Self::Lotto740,
        Self::TicTacToe,
        Self::Lotto638,
        Self::BingoBingo,
    ];

    /// Returns static metadata describing display name and number-selection rules.
    pub const fn metadata(self) -> LotteryGameMetadata {
        metadata_for_game(self)
    }

    /// Returns metadata with `display_name` chosen by enum language.
    pub fn metadata_with_language(self, language: LotteryDisplayLanguage) -> LotteryGameMetadata {
        self.metadata().with_display_language(language)
    }

    /// Returns the allowed query month range for this game in `YYYY-MM` format.
    ///
    /// The max month is capped to current UTC month for active fifth-term games.
    pub fn query_month_range(self) -> LotteryGameQueryRange {
        let (start, end) = game_query_month_bounds(self);
        LotteryGameQueryRange {
            min_month: start.to_yyyy_mm(),
            max_month: end.to_yyyy_mm(),
        }
    }

    /// Returns remote API query-parameter support for this game endpoint.
    pub const fn remote_query_param_support(self) -> RemoteQueryParamSupport {
        query::remote::remote_query_param_support(self)
    }

    /// Parses CLI/user aliases into a supported game.
    ///
    /// Parsing is ASCII case-insensitive and accepts both display-oriented names
    /// and short numeric aliases used by Taiwan Lottery and the C API.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "super-lotto638" | "superlotto638" | "5134" => Some(Self::SuperLotto638),
            "lotto649" | "5118" => Some(Self::Lotto649),
            "daily539" | "5120" => Some(Self::Daily539),
            "3d" | "2108" => Some(Self::Lotto3D),
            "4d" | "2109" => Some(Self::Lotto4D),
            "49m6" | "1121" => Some(Self::Lotto49M6),
            "39m5" | "1197" => Some(Self::Lotto39M5),
            "38m6" | "5122" => Some(Self::Lotto38M6),
            "1224" | "5290" => Some(Self::Lotto1224),
            "740" | "2300" => Some(Self::Lotto740),
            "tic-tac-toe" | "tictactoe" | "2400" => Some(Self::TicTacToe),
            "638" | "2500" => Some(Self::Lotto638),
            "bingo-bingo" | "bingobingo" | "bingo_bingo" | "1102" => Some(Self::BingoBingo),
            _ => None,
        }
    }

    /// Maps the integer codes used by the FFI and C API back to a game.
    pub const fn from_code(code: i32) -> Option<Self> {
        match code {
            0 => Some(Self::SuperLotto638),
            1 => Some(Self::Lotto649),
            2 => Some(Self::Daily539),
            3 => Some(Self::Lotto3D),
            4 => Some(Self::Lotto4D),
            5 => Some(Self::Lotto49M6),
            6 => Some(Self::Lotto39M5),
            7 => Some(Self::Lotto38M6),
            8 => Some(Self::Lotto1224),
            9 => Some(Self::Lotto740),
            10 => Some(Self::TicTacToe),
            11 => Some(Self::Lotto638),
            12 => Some(Self::BingoBingo),
            _ => None,
        }
    }

    pub(crate) fn path(self) -> &'static str {
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
            Self::BingoBingo => "/Lottery/BingoResult",
        }
    }

    pub(crate) fn history_session_path(self) -> Option<&'static str> {
        match self {
            Self::Lotto3D => Some("/Lottery/3DHistoryResult"),
            Self::Lotto4D => Some("/Lottery/4DHistoryResult"),
            _ => None,
        }
    }
}

impl std::str::FromStr for LotteryGame {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl TryFrom<i32> for LotteryGame {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        Self::from_code(value).ok_or(())
    }
}

/// Query parameters for historical lottery draw searches.
///
/// Use builder methods to construct queries:
/// - [`by_period`](HistoryDrawQuery::by_period) - Query by a specific period
/// - [`by_month`](HistoryDrawQuery::by_month) - Query a single month
/// - [`by_month_range`](HistoryDrawQuery::by_month_range) - Query a date range
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HistoryDrawQuery {
    pub period: Option<String>,
    pub month: Option<String>,
    pub end_month: Option<String>,
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
            .ok_or_else(|| std::io::Error::other("month is required when period is empty"))?;
        let end_month = self
            .end_month
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(month);

        Ok(("", month, end_month))
    }
}

/// A single lottery draw result from historical data.
///
/// Contains the draw period/date and corresponding numbers. The `numbers` field
/// contains both base numbers (in draw order) and sorted numbers when available.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HistoryDrawItem {
    pub period: String,
    pub date: Option<String>,
    pub redeemable_date: Option<String>,
    pub numbers: SortedDrawNumbers,
}

/// Paginated result set from a historical lottery draw query.
///
/// Contains the total number of matching results and a collection of individual draw items.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HistoryDrawPage {
    pub total_size: usize,
    pub items: Vec<HistoryDrawItem>,
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
            if let Some(extension) = extension_from_magic_bytes(&file_bytes) {
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

fn build_http_client() -> Result<reqwest::blocking::Client, DownloadError> {
    reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(DownloadError::from)
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

pub(crate) fn download_api_doc_impl(
    output_dir: impl AsRef<Path>,
) -> Result<PathBuf, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    let (_, path) = download_api_doc_with_client(&client, output_dir)?;
    Ok(path)
}

pub(crate) fn download_dataset_impl(
    output_dir: impl AsRef<Path>,
    dataset_code: &str,
) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    download_dataset_with_client(&client, output_dir, dataset_code)
}

pub(crate) fn download_history_draw_from_gov_data_impl(
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    download_dataset_with_client(&client, output_dir, HISTORY_DRAW_CODE)
}

pub(crate) fn download_history_draw_from_taiwan_lottery_impl(
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, DownloadError> {
    let output_dir = output_dir.as_ref();
    let client = build_http_client()?;
    download_history_draw_from_taiwan_lottery_with_client(&client, output_dir)
}

pub(crate) fn download_history_draw_impl(
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, DownloadError> {
    match download_history_draw_from_gov_data_impl(output_dir.as_ref()) {
        Ok(files) => Ok(files),
        Err(DownloadError::Http(_)) => {
            download_history_draw_from_taiwan_lottery_impl(output_dir.as_ref())
        }
        Err(err) => Err(err),
    }
}

pub(crate) fn download_all_impl(
    output_dir: impl AsRef<Path>,
) -> Result<Vec<PathBuf>, DownloadError> {
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
            0xE4u8, 0xB8, 0xAD, 0xE6, 0x96, 0x87, 0xE5, 0xA0, 0xB1, 0xE8, 0xA1, 0xA8, b'.', b'p',
            b'd', b'f',
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
        headers.insert(
            CONTENT_TYPE,
            "application/pdf".parse().expect("valid content type"),
        );

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

        let extract_dir =
            std::env::temp_dir().join(format!("taiwan-lottery-zip-test-{}", std::process::id()));
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

        let page =
            query::remote::parse_history_draw_page(&sample).expect("parse history draw page");
        assert_eq!(page.total_size, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].period, "112000116");
        assert_eq!(
            page.items[0].numbers.base.numbers,
            vec![31, 46, 11, 39, 23, 1, 17]
        );
        assert_eq!(
            page.items[0].numbers.sorted,
            Some(vec![1, 11, 23, 31, 39, 46, 17])
        );
    }

    #[test]
    fn parse_history_draw_page_accepts_empty_result_arrays() {
        let sample = serde_json::json!({
            "totalSize": 0,
            "lotto638Res": []
        });

        let page =
            query::remote::parse_history_draw_page(&sample).expect("parse empty history draw page");
        assert_eq!(page.total_size, 0);
        assert!(page.items.is_empty());
    }

    #[test]
    fn history_draw_query_requires_month_when_period_is_empty() {
        let query = HistoryDrawQuery::default();
        let err = query
            .normalized_params()
            .expect_err("must fail without period or month");
        assert!(matches!(err, DownloadError::Io(_)));
    }

    #[test]
    fn validate_query_range_rejects_out_of_term_month_for_lotto_1224() {
        let query = HistoryDrawQuery::by_month("2013-12");
        let err = query::remote::validate_query_range_for_game(LotteryGame::Lotto1224, &query)
            .expect_err("1224 should not allow third-term month");
        assert!(matches!(err, DownloadError::Io(_)));
    }

    #[test]
    fn validate_query_range_rejects_out_of_term_month_for_tic_tac_toe() {
        let query = HistoryDrawQuery::by_month("2014-01");
        let err = query::remote::validate_query_range_for_game(LotteryGame::TicTacToe, &query)
            .expect_err("tic-tac-toe should not allow fourth-term month");
        assert!(matches!(err, DownloadError::Io(_)));
    }

    #[test]
    fn validate_query_range_rejects_out_of_term_period_for_lotto_740() {
        let query = HistoryDrawQuery::by_period("113000001");
        let err = query::remote::validate_query_range_for_game(LotteryGame::Lotto740, &query)
            .expect_err("740 should not allow fifth-term period");
        assert!(matches!(err, DownloadError::Io(_)));
    }

    #[test]
    fn validate_query_range_accepts_third_to_fourth_overlap_game() {
        let query = HistoryDrawQuery::by_month("2023-12");
        query::remote::validate_query_range_for_game(LotteryGame::Lotto38M6, &query)
            .expect("38M6 should allow fourth-term month");
    }

    #[test]
    fn validate_query_range_rejects_future_month_for_fifth_active_game() {
        let now = query::remote::current_utc_year_month();
        let (future_year, future_month) = if now.month == 12 {
            (now.year + 1, 1)
        } else {
            (now.year, now.month + 1)
        };
        let query = HistoryDrawQuery::by_month(format!("{future_year:04}-{future_month:02}"));
        let err = query::remote::validate_query_range_for_game(LotteryGame::Lotto649, &query)
            .expect_err("lotto649 should not allow future month");
        assert!(matches!(err, DownloadError::Io(_)));
    }

    #[test]
    fn lottery_game_query_month_range_is_exposed_for_ui() {
        let range = LotteryGame::Lotto1224.query_month_range();
        assert_eq!(range.min_month, "2014-01");
        assert_eq!(range.max_month, "2023-12");
    }

    #[test]
    fn lottery_game_query_month_range_caps_active_game_to_current_month() {
        let range = LotteryGame::Lotto649.query_month_range();
        let now = query::remote::current_utc_year_month();
        assert_eq!(range.min_month, "2007-01");
        assert_eq!(range.max_month, format!("{:04}-{:02}", now.year, now.month));
    }

    #[test]
    fn lottery_game_query_month_range_for_bingo_bingo_uses_2024_start() {
        let range = LotteryGame::BingoBingo.query_month_range();
        let now = query::remote::current_utc_year_month();
        assert_eq!(range.min_month, "2024-01");
        assert_eq!(range.max_month, format!("{:04}-{:02}", now.year, now.month));
    }

    #[test]
    fn remote_query_param_support_matches_empirical_behavior() {
        let non_bingo = LotteryGame::Lotto649.remote_query_param_support();
        assert!(non_bingo.month);
        assert!(non_bingo.end_month);
        assert!(!non_bingo.open_date);
        assert!(non_bingo.period);

        let bingo = LotteryGame::BingoBingo.remote_query_param_support();
        assert!(!bingo.month);
        assert!(!bingo.end_month);
        assert!(bingo.open_date);
        assert!(!bingo.period);
    }

    #[test]
    fn history_game_metadata_exposes_ui_fields() {
        let metadata = LotteryGame::Lotto649.metadata();
        assert_eq!(metadata.display_name, "Lotto 649");
        assert_eq!(metadata.display_name_english, "Lotto 649");
        assert_eq!(metadata.display_name_chinese, "大樂透");
        assert_eq!(
            metadata.number_rule,
            "6 numbers from 1-49, plus 1 bonus number from 1-49"
        );
        assert_eq!(metadata.number_ranges.len(), 2);
        assert_eq!(metadata.number_ranges[0].name, "main");
        assert_eq!(metadata.number_ranges[0].picks, 6);
        assert_eq!(metadata.number_ranges[0].min, 1);
        assert_eq!(metadata.number_ranges[0].max, 49);
        assert!(!metadata.number_ranges[0].allow_repeat);
    }

    #[test]
    fn history_game_metadata_supports_language_input() {
        let default_metadata =
            LotteryGame::Lotto649.metadata_with_language(LotteryDisplayLanguage::English);
        assert_eq!(default_metadata.display_name, "Lotto 649");

        let chinese_metadata =
            LotteryGame::Lotto649.metadata_with_language(LotteryDisplayLanguage::Chinese);
        assert_eq!(chinese_metadata.display_name, "大樂透");
    }

    #[test]
    fn lottery_game_parse_supports_aliases() {
        assert_eq!(LotteryGame::parse("lotto649"), Some(LotteryGame::Lotto649));
        assert_eq!(LotteryGame::parse("5118"), Some(LotteryGame::Lotto649));
        assert_eq!(
            LotteryGame::parse("tic-tac-toe"),
            Some(LotteryGame::TicTacToe)
        );
        assert_eq!(
            LotteryGame::parse("bingo-bingo"),
            Some(LotteryGame::BingoBingo)
        );
        assert_eq!(LotteryGame::parse("1102"), Some(LotteryGame::BingoBingo));
        assert_eq!(LotteryGame::parse("unknown"), None);
    }

    #[test]
    fn lottery_game_from_code_matches_ffi_codes() {
        assert_eq!(LotteryGame::from_code(0), Some(LotteryGame::SuperLotto638));
        assert_eq!(LotteryGame::from_code(11), Some(LotteryGame::Lotto638));
        assert_eq!(LotteryGame::from_code(12), Some(LotteryGame::BingoBingo));
        assert_eq!(LotteryGame::from_code(99), None);
    }

    #[test]
    fn lottery_game_from_str_matches_parse() {
        use std::str::FromStr as _;

        assert_eq!(LotteryGame::from_str("lotto649"), Ok(LotteryGame::Lotto649));
        assert_eq!(LotteryGame::from_str("5118"), Ok(LotteryGame::Lotto649));
        assert_eq!(LotteryGame::from_str("invalid"), Err(()));
    }

    #[test]
    fn lottery_game_try_from_i32_matches_from_code() {
        assert_eq!(LotteryGame::try_from(0), Ok(LotteryGame::SuperLotto638));
        assert_eq!(LotteryGame::try_from(11), Ok(LotteryGame::Lotto638));
        assert_eq!(LotteryGame::try_from(12), Ok(LotteryGame::BingoBingo));
        assert_eq!(LotteryGame::try_from(-1), Err(()));
    }

    #[test]
    fn history_game_all_lists_every_supported_game() {
        assert_eq!(LotteryGame::ALL.len(), 13);
        assert!(LotteryGame::ALL.contains(&LotteryGame::TicTacToe));
        assert!(LotteryGame::ALL.contains(&LotteryGame::SuperLotto638));
        assert!(LotteryGame::ALL.contains(&LotteryGame::BingoBingo));
    }

    #[test]
    fn local_3d_history_draw_uses_numbers_draw() {
        let root = std::env::temp_dir().join(format!(
            "taiwan-lottery-history-local-3d-test-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("cleanup old temp dir");
        }

        let game_dir = root.join("D423F").join("2022");
        fs::create_dir_all(&game_dir).expect("create game dir");
        let file = game_dir.join("3星彩_2022.csv");
        fs::write(
            &file,
            "遊戲名稱,期別,開獎日期,獎號1,獎號2,獎號3
3星彩,111000155,2022/06/30,5,9,3
",
        )
        .expect("write csv");

        let query = HistoryDrawQuery::by_period("111000155");
        let page =
            query_history_draw(&root, LotteryGame::Lotto3D, query).expect("query local 3d data");
        assert_eq!(page.total_size, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.items[0].numbers.base.numbers, vec![5, 9, 3]);
        assert_eq!(page.items[0].numbers.sorted, None);

        fs::remove_dir_all(&root).expect("cleanup temp dir");
    }

    #[test]
    fn lotto38m6_does_not_include_lotto649_addon_prefix() {
        let prefixes = query::local::history_game_file_prefixes(LotteryGame::Lotto38M6);
        assert!(prefixes.contains(&"38樂合彩_"));
        assert!(!prefixes.contains(&"大樂透加開獎項_"));
    }

    #[test]
    fn lotto3d_and_4d_use_numeric_prefixes_only() {
        let p3d = query::local::history_game_file_prefixes(LotteryGame::Lotto3D);
        let p4d = query::local::history_game_file_prefixes(LotteryGame::Lotto4D);
        assert_eq!(p3d, &["3星彩_"]);
        assert_eq!(p4d, &["4星彩_"]);
    }

    #[test]
    fn bingo_family_uses_strict_prefixes() {
        assert_eq!(
            query::local::history_game_file_prefixes(LotteryGame::Lotto1224),
            &["雙贏彩_"]
        );
        assert_eq!(
            query::local::history_game_file_prefixes(LotteryGame::Lotto740),
            &["大福彩_"]
        );
        assert_eq!(
            query::local::history_game_file_prefixes(LotteryGame::TicTacToe),
            &["樂線九宮格_"]
        );
        assert_eq!(
            query::local::history_game_file_prefixes(LotteryGame::Lotto638),
            &["6_38樂透彩_"]
        );
        assert_eq!(
            query::local::history_game_file_prefixes(LotteryGame::BingoBingo),
            &["賓果賓果_"]
        );
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
        let page =
            query_history_draw(&root, LotteryGame::Lotto649, query).expect("query local data");
        assert_eq!(page.total_size, 1);
        assert_eq!(page.items.len(), 1);
        assert_eq!(
            page.items[0].numbers.base.numbers,
            vec![3, 7, 16, 19, 40, 42, 12]
        );
        assert_eq!(
            page.items[0].numbers.sorted,
            Some(vec![3, 7, 16, 19, 40, 42, 12])
        );

        fs::remove_dir_all(&root).expect("cleanup temp dir");
    }
}
