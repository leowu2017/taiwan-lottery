use std::path::Path;

use crate::{DownloadError, HistoryDrawPage, HistoryDrawQuery, HistoryGame};

pub fn query_history_draw(
    output_dir: impl AsRef<Path>,
    game: HistoryGame,
    query: HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    crate::get_history_draw_impl(output_dir, game, query)
}

pub fn query_history_draw_from_taiwan_lottory(
    game: HistoryGame,
    query: HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    crate::get_history_draw_from_taiwan_lottory_impl(game, query)
}
