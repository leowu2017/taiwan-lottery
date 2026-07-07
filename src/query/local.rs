use std::fs;
use std::path::{Path, PathBuf};

use crate::query::remote::validate_query_range_for_game;
use crate::{
    DownloadError, HistoryDrawItem, HistoryDrawPage, HistoryDrawQuery, LotteryGame,
    SortedDrawNumbers,
};

const HISTORY_DRAW_CODE: &str = "D423F";

#[derive(Debug, Clone)]
struct LocalHistoryDrawRecord {
    period: String,
    date: Option<String>,
    numbers_sorted: Vec<i32>,
}

pub(crate) fn history_game_file_prefixes(game: LotteryGame) -> &'static [&'static str] {
    // Keep local file matching strict so similarly named games do not bleed into each other.
    match game {
        LotteryGame::SuperLotto638 => &["威力彩_"],
        LotteryGame::Lotto649 => &["大樂透_"],
        LotteryGame::Daily539 => &["今彩539_"],
        LotteryGame::Lotto3D => &["3星彩_"],
        LotteryGame::Lotto4D => &["4星彩_"],
        LotteryGame::Lotto49M6 => &["49樂合彩_"],
        LotteryGame::Lotto39M5 => &["39樂合彩_"],
        LotteryGame::Lotto38M6 => &["38樂合彩_"],
        LotteryGame::Lotto1224 => &["雙贏彩_"],
        LotteryGame::Lotto740 => &["大福彩_"],
        LotteryGame::TicTacToe => &["樂線九宮格_"],
        LotteryGame::Lotto638 => &["6_38樂透彩_"],
        LotteryGame::BingoBingo => &["賓果賓果_"],
    }
}

fn resolve_history_data_root(output_dir: &Path) -> Result<PathBuf, DownloadError> {
    // Accept either the repository data root or a direct D423F directory path.
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

fn parse_date_month(date: &str) -> Option<String> {
    let normalized = date.trim().replace('/', "-");
    if normalized.len() >= 7 {
        Some(normalized[..7].to_string())
    } else {
        None
    }
}

fn extract_draw_numbers(headers: &csv::StringRecord, record: &csv::StringRecord) -> Vec<i32> {
    // Taiwan Lottery CSVs use a mix of primary/bonus column names across games.
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
    let date_index = headers
        .iter()
        .position(|header| header.trim() == "開獎日期");

    let mut records = Vec::new();
    for row in reader.records() {
        let row = row?;
        let period = row.get(period_index).unwrap_or_default().trim().to_string();
        if period.is_empty() {
            continue;
        }

        let numbers_sorted = extract_draw_numbers(&headers, &row);
        if numbers_sorted.is_empty() {
            continue;
        }

        let date = date_index
            .and_then(|index| row.get(index))
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned);

        records.push(LocalHistoryDrawRecord {
            period,
            date,
            numbers_sorted,
        });
    }

    Ok(records)
}

pub(crate) fn query_history_draw_from_downloaded_data(
    output_dir: &Path,
    game: LotteryGame,
    query: &HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    // Local CSVs are aggregated across years, so filter after collecting and dedup by period.
    validate_query_range_for_game(game, query)?;
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
                .date
                .as_deref()
                .and_then(parse_date_month)
                .is_some_and(|value| value == month)
        });
    }

    all_records.sort_by(|left, right| right.period.cmp(&left.period));
    all_records.dedup_by(|left, right| left.period == right.period);

    let total_size = all_records.len();
    let items = all_records
        .iter()
        .map(|record| {
            let (base_numbers, sorted_numbers) = match game {
                LotteryGame::Lotto3D | LotteryGame::Lotto4D => {
                    (record.numbers_sorted.clone(), None)
                }
                _ => (
                    record.numbers_sorted.clone(),
                    Some(record.numbers_sorted.clone()),
                ),
            };

            HistoryDrawItem {
                period: record.period.clone(),
                date: record.date.clone(),
                redeemable_date: None,
                numbers: SortedDrawNumbers::new(base_numbers, sorted_numbers),
            }
        })
        .collect();

    Ok(HistoryDrawPage { total_size, items })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_date_month_supports_dash_and_slash() {
        assert_eq!(parse_date_month("2026-07-07"), Some("2026-07".to_string()));
        assert_eq!(parse_date_month("2026/07/07"), Some("2026-07".to_string()));
        assert_eq!(parse_date_month("2026"), None);
    }

    #[test]
    fn bingo_and_lotto638_prefixes_are_strictly_separated() {
        assert_eq!(
            history_game_file_prefixes(LotteryGame::Lotto638),
            &["6_38樂透彩_"]
        );
        assert_eq!(
            history_game_file_prefixes(LotteryGame::BingoBingo),
            &["賓果賓果_"]
        );
    }
}
