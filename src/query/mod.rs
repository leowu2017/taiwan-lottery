pub(crate) mod common;
mod local;
mod remote;

use std::path::Path;

use crate::{DownloadError, HistoryDrawPage, HistoryDrawQuery, LotteryGame};

pub(crate) use remote::remote_query_param_support;

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
pub fn query_history_draw(
    output_dir: impl AsRef<Path>,
    game: LotteryGame,
    query: HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    local::query_history_draw_from_downloaded_data(output_dir.as_ref(), game, &query)
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
pub fn query_history_draw_from_taiwan_lottery(
    game: LotteryGame,
    query: HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    remote::query_history_draw_from_taiwan_lottery(game, &query)
}
