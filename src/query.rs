use std::path::Path;

use crate::{DownloadError, HistoryDrawPage, HistoryDrawQuery, HistoryGame};

pub fn query_history_draw(
    output_dir: impl AsRef<Path>,
    game: HistoryGame,
    query: HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    crate::query_history_draw_impl(output_dir, game, query)
}

pub fn query_history_draw_from_taiwan_lottery(
    game: HistoryGame,
    query: HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    crate::query_history_draw_from_taiwan_lottery_impl(game, query)
}
