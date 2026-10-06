use std::fs;
use std::path::{Path, PathBuf};

use super::common::{
    parse_date_year_month, parse_date_year_month_day, period_newest_first,
    validate_query_range_for_game, YearMonth, YearMonthDay,
};
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
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("history data directory not found: {}", d423f_dir.display()),
        )
        .into())
    }
}

fn year_from_path(path: &Path) -> Option<i32> {
    // Only four-digit names are AD years; older archives nest ROC-year folders such as `2007/96`.
    path.file_name()
        .and_then(|value| value.to_str())
        .filter(|value| value.len() == 4)
        .and_then(|value| value.parse::<i32>().ok())
}

fn should_descend_into_dir(path: &Path, year_range: Option<(i32, i32)>) -> bool {
    match (year_range, year_from_path(path)) {
        (Some((start, end)), Some(actual)) => (start..=end).contains(&actual),
        _ => true,
    }
}

fn collect_history_csv_files(
    root: &Path,
    prefixes: &[&str],
    year_range: Option<(i32, i32)>,
    output: &mut Vec<PathBuf>,
) -> Result<(), DownloadError> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if should_descend_into_dir(&path, year_range) {
                collect_history_csv_files(&path, prefixes, year_range, output)?;
            }
            continue;
        }

        if !path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| value.eq_ignore_ascii_case("csv"))
        {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();

        if !prefixes.iter().any(|prefix| file_name.starts_with(prefix)) {
            continue;
        }

        if let Some((start, end)) = year_range {
            let file_matches_year =
                (start..=end).any(|year| file_name.contains(&format!("_{year}.csv")));
            let path_matches_year = path.ancestors().any(|ancestor| {
                year_from_path(ancestor).is_some_and(|year| (start..=end).contains(&year))
            });

            // D423F is grouped by year directories; accept either folder-based
            // or filename-based year layout.
            if !file_matches_year && !path_matches_year {
                continue;
            }
        }

        output.push(path);
    }

    Ok(())
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
            DownloadError::data(format!(
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
    let period = query.period.as_deref().unwrap_or("").trim();
    let open_date = query
        .open_date
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let root = resolve_history_data_root(output_dir)?;

    let prefixes = history_game_file_prefixes(game);
    let (month_range, day_filter) = if !period.is_empty() {
        (None, None)
    } else if let Some(open_date) = open_date {
        let day = YearMonthDay::parse_yyyy_mm_dd(open_date)?;
        (Some((day.to_year_month(), day.to_year_month())), Some(day))
    } else {
        let (_, month, end_month) = query.normalized_params()?;
        let range = (
            YearMonth::parse_yyyy_mm(month)?,
            YearMonth::parse_yyyy_mm(end_month)?,
        );
        (Some(range), None)
    };
    let year_range = month_range.map(|(start, end)| (start.year, end.year));

    let mut csv_files = Vec::new();
    collect_history_csv_files(&root, prefixes, year_range, &mut csv_files)?;

    let mut all_records = Vec::new();
    for file_path in csv_files {
        let mut file_records = parse_history_csv_file(&file_path)?;
        all_records.append(&mut file_records);
    }

    if let Some((start, end)) = month_range {
        all_records.retain(|record| {
            record
                .date
                .as_deref()
                .and_then(parse_date_year_month)
                .is_some_and(|value| value >= start && value <= end)
        });
    } else {
        all_records.retain(|record| record.period == period);
    }
    if let Some(day) = day_filter {
        all_records.retain(|record| {
            record.date.as_deref().and_then(parse_date_year_month_day) == Some(day)
        });
    }

    all_records.sort_by(|left, right| period_newest_first(&left.period, &right.period));
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
    use crate::query_history_draw;
    use std::fs;

    #[test]
    fn year_path_helpers_match_numeric_year_dirs_only() {
        assert_eq!(year_from_path(Path::new("/tmp/2024")), Some(2024));
        assert_eq!(year_from_path(Path::new("/tmp/not-a-year")), None);
        assert_eq!(year_from_path(Path::new("/tmp/96")), None);
        assert!(should_descend_into_dir(
            Path::new("/tmp/2024"),
            Some((2024, 2024))
        ));
        assert!(should_descend_into_dir(
            Path::new("/tmp/2024"),
            Some((2023, 2025))
        ));
        assert!(!should_descend_into_dir(
            Path::new("/tmp/2025"),
            Some((2024, 2024))
        ));
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
        let prefixes = history_game_file_prefixes(LotteryGame::Lotto38M6);
        assert!(prefixes.contains(&"38樂合彩_"));
        assert!(!prefixes.contains(&"大樂透加開獎項_"));
    }

    #[test]
    fn lotto3d_and_4d_use_numeric_prefixes_only() {
        let p3d = history_game_file_prefixes(LotteryGame::Lotto3D);
        let p4d = history_game_file_prefixes(LotteryGame::Lotto4D);
        assert_eq!(p3d, &["3星彩_"]);
        assert_eq!(p4d, &["4星彩_"]);
    }

    #[test]
    fn bingo_family_uses_strict_prefixes() {
        assert_eq!(
            history_game_file_prefixes(LotteryGame::Lotto1224),
            &["雙贏彩_"]
        );
        assert_eq!(
            history_game_file_prefixes(LotteryGame::Lotto740),
            &["大福彩_"]
        );
        assert_eq!(
            history_game_file_prefixes(LotteryGame::TicTacToe),
            &["樂線九宮格_"]
        );
        assert_eq!(
            history_game_file_prefixes(LotteryGame::Lotto638),
            &["6_38樂透彩_"]
        );
        assert_eq!(
            history_game_file_prefixes(LotteryGame::BingoBingo),
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

    #[test]
    fn month_range_query_spans_months_and_years() {
        let root = std::env::temp_dir().join(format!(
            "taiwan-lottery-history-local-range-test-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("cleanup old temp dir");
        }

        let header = "遊戲名稱,期別,開獎日期,獎號1,獎號2,獎號3,獎號4,獎號5,獎號6,特別號\n";
        for (year, rows) in [
            (
                2025,
                "大樂透,114000100,2025/11/28,1,2,3,4,5,6,7\n大樂透,114000104,2025/12/12,1,2,3,4,5,6,7\n",
            ),
            (
                2026,
                "大樂透,115000001,2026/01/02,1,2,3,4,5,6,7\n大樂透,115000010,2026/02/03,1,2,3,4,5,6,7\n",
            ),
        ] {
            let dir = root.join("D423F").join(year.to_string());
            fs::create_dir_all(&dir).expect("create year dir");
            fs::write(dir.join(format!("大樂透_{year}.csv")), format!("{header}{rows}"))
                .expect("write csv");
        }

        let query = HistoryDrawQuery::by_month_range("2025-12", "2026-01");
        let page = query_history_draw(&root, LotteryGame::Lotto649, query).expect("range query");
        let periods: Vec<&str> = page.items.iter().map(|item| item.period.as_str()).collect();
        assert_eq!(periods, vec!["115000001", "114000104"]);
        assert_eq!(page.total_size, 2);

        fs::remove_dir_all(&root).expect("cleanup temp dir");
    }

    #[test]
    fn query_finds_files_in_nested_roc_year_folders() {
        let root = std::env::temp_dir().join(format!(
            "taiwan-lottery-history-local-roc-test-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("cleanup old temp dir");
        }

        let dir = root.join("D423F").join("2008").join("97");
        fs::create_dir_all(&dir).expect("create nested dir");
        fs::write(
            dir.join("大樂透_2008.csv"),
            "遊戲名稱,期別,開獎日期,獎號1,獎號2,獎號3,獎號4,獎號5,獎號6,特別號\n大樂透,97000010,2008/02/05,1,2,3,4,5,6,7\n",
        )
        .expect("write csv");

        let query = HistoryDrawQuery::by_month("2008-02");
        let page = query_history_draw(&root, LotteryGame::Lotto649, query).expect("query");
        assert_eq!(page.total_size, 1);
        assert_eq!(page.items[0].period, "97000010");

        fs::remove_dir_all(&root).expect("cleanup temp dir");
    }

    #[test]
    fn open_date_query_returns_only_that_day_for_bingo_bingo() {
        let root = std::env::temp_dir().join(format!(
            "taiwan-lottery-history-local-bingo-test-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("cleanup old temp dir");
        }

        let dir = root.join("D423F").join("2024");
        fs::create_dir_all(&dir).expect("create year dir");
        fs::write(
            dir.join("賓果賓果_2024.csv"),
            "遊戲名稱,期別,開獎日期,獎號1,獎號2,獎號3\n賓果賓果,113000001,2024/01/01,1,2,3\n賓果賓果,113000002,2024/01/02,4,5,6\n賓果賓果,113000003,2024/01/02,7,8,9\n",
        )
        .expect("write csv");

        let query = HistoryDrawQuery::by_open_date("2024-01-02");
        let page = query_history_draw(&root, LotteryGame::BingoBingo, query).expect("query");
        let periods: Vec<&str> = page.items.iter().map(|item| item.period.as_str()).collect();
        assert_eq!(periods, vec!["113000003", "113000002"]);

        fs::remove_dir_all(&root).expect("cleanup temp dir");
    }

    #[test]
    fn non_ascii_date_does_not_panic() {
        let root = std::env::temp_dir().join(format!(
            "taiwan-lottery-history-local-nonascii-test-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("cleanup old temp dir");
        }

        let dir = root.join("D423F").join("2026");
        fs::create_dir_all(&dir).expect("create year dir");
        fs::write(
            dir.join("大樂透_2026.csv"),
            "遊戲名稱,期別,開獎日期,獎號1,獎號2,獎號3,獎號4,獎號5,獎號6,特別號\n大樂透,115000001,二〇二六年一月,1,2,3,4,5,6,7\n",
        )
        .expect("write csv");

        let query = HistoryDrawQuery::by_month("2026-01");
        let page = query_history_draw(&root, LotteryGame::Lotto649, query).expect("query");
        assert_eq!(page.total_size, 0);

        fs::remove_dir_all(&root).expect("cleanup temp dir");
    }
}
