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

pub mod download;
mod draw;
mod errors;
mod ffi;
mod numbers;
mod query;
mod rule;

use query::common::game_query_date_bounds;
use query::common::game_query_month_bounds;
use query::remote_query_param_support;
use rule::metadata_for_game;

pub use download::{
    build_csv_url, download_all, download_api_doc, download_dataset, parse_codes_from_api_docs,
};
/// Downloads history draw files from FinancialPlanning OpenData (`D423F`).
#[deprecated(note = "use taiwan_lottery::download::gaze::download_history_draw")]
pub fn download_history_draw(
    output_dir: impl AsRef<std::path::Path>,
) -> Result<Vec<std::path::PathBuf>, DownloadError> {
    download::gaze::download_history_draw(output_dir)
}
/// Downloads history draw files from Taiwan Lottery yearly ZIP API.
#[deprecated(note = "use taiwan_lottery::download::tlc::download_history_draw")]
pub fn download_history_draw_from_taiwan_lottery(
    output_dir: impl AsRef<std::path::Path>,
) -> Result<Vec<std::path::PathBuf>, DownloadError> {
    download::tlc::download_history_draw(output_dir)
}
pub use draw::{draw_by_game, DrawResult};
pub use errors::DownloadError;
pub use numbers::{
    BingoBingoNumbers, BonusDrawNumbers, Daily539Numbers, DrawNumbers, Lotto1224Numbers,
    Lotto38M6Numbers, Lotto39M5Numbers, Lotto3DNumbers, Lotto49M6Numbers, Lotto4DNumbers,
    Lotto638Numbers, Lotto649Numbers, Lotto740Numbers, SortedDrawNumbers, SuperLotto638Numbers,
    TicTacToeNumbers,
};
pub use query::{query_history_draw, query_history_draw_from_taiwan_lottery};

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

/// Queryable date range for a game in `YYYY-MM-DD` format.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LotteryGameDateQueryRange {
    /// Earliest supported query date (inclusive).
    pub min_date: String,
    /// Latest supported query date (inclusive).
    pub max_date: String,
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

    /// Returns the allowed query date range for this game in `YYYY-MM-DD` format.
    ///
    /// The max date is capped to current UTC date for active fifth-term games.
    pub fn query_date_range(self) -> LotteryGameDateQueryRange {
        let (start, end) = game_query_date_bounds(self);
        LotteryGameDateQueryRange {
            min_date: start.to_yyyy_mm_dd(),
            max_date: end.to_yyyy_mm_dd(),
        }
    }

    /// Returns remote API query-parameter support for this game endpoint.
    pub const fn remote_query_param_support(self) -> RemoteQueryParamSupport {
        remote_query_param_support(self)
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
/// - [`by_open_date`](HistoryDrawQuery::by_open_date) - Query by a specific open date (Bingo)
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct HistoryDrawQuery {
    pub period: Option<String>,
    pub month: Option<String>,
    pub end_month: Option<String>,
    pub open_date: Option<String>,
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

    pub fn by_open_date(open_date: impl Into<String>) -> Self {
        Self {
            open_date: Some(open_date.into()),
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

#[cfg(test)]
mod tests {
    use super::*;

    mod history_draw_query_tests {
        use super::*;

        #[test]
        fn history_draw_query_requires_month_when_period_is_empty() {
            let query = HistoryDrawQuery::default();
            let err = query
                .normalized_params()
                .expect_err("must fail without period or month");
            assert!(matches!(err, DownloadError::Io(_)));
        }
    }

    mod lottery_game_tests {
        use super::*;

        #[test]
        fn lottery_game_query_month_range_is_exposed_for_ui() {
            let range = LotteryGame::Lotto1224.query_month_range();
            assert_eq!(range.min_month, "2018-04");
            assert_eq!(range.max_month, "2023-12");
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
    }
}
