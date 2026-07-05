use std::path::Path;

use crate::{DownloadError, HistoryDrawPage, HistoryDrawQuery, LotteryGame};

/// Query historical lottery draw results from locally downloaded data.
///
/// Reads history draw data that was previously downloaded by [`crate::download_history_draw`]
/// and searches for matching draws.
///
/// # Arguments
/// * `output_dir` - Root directory containing downloaded data (should have `output_dir/D423F/` structure)
/// * `game` - The lottery game to query
/// * `query` - Query parameters (period, month, or month range)
///
/// # Returns
/// A [`HistoryDrawPage`] containing matching results and total count.
///
/// # Errors
/// Returns [`DownloadError`] if files cannot be read or parsed.
///
/// # Example
/// ```ignore
/// use taiwan_lottery::{query_history_draw, HistoryDrawQuery, LotteryGame};
///
/// let query = HistoryDrawQuery::by_month("2023-12");
/// let results = query_history_draw("./data", LotteryGame::Lotto649, query)?;
/// for item in results.items {
///     println!("Period {}: {:?}", item.period, item.numbers.base.numbers);
/// }
/// ```
pub fn query_history_draw(
    output_dir: impl AsRef<Path>,
    game: LotteryGame,
    query: HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    crate::query_history_draw_impl(output_dir, game, query)
}

/// Query historical lottery draw results directly from Taiwan Lottery API.
///
/// This function calls the Taiwan Lottery web API directly without requiring
/// pre-downloaded data.
///
/// # Arguments
/// * `game` - The lottery game to query
/// * `query` - Query parameters (period, month, or month range)
///
/// # Returns
/// A [`HistoryDrawPage`] containing matching results and total count.
///
/// # Errors
/// Returns [`DownloadError`] if the API request or response parsing fails.
///
/// # Example
/// ```ignore
/// use taiwan_lottery::{query_history_draw_from_taiwan_lottery, HistoryDrawQuery, LotteryGame};
///
/// let query = HistoryDrawQuery::by_month("2026-01");
/// let results = query_history_draw_from_taiwan_lottery(LotteryGame::Lotto649, query)?;
/// ```
pub fn query_history_draw_from_taiwan_lottery(
    game: LotteryGame,
    query: HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    crate::query_history_draw_from_taiwan_lottery_impl(game, query)
}
