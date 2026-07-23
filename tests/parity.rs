use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use taiwan_lottery::{
    query_history_draw, query_history_draw_from_taiwan_lottery, DownloadError, HistoryDrawPage,
    HistoryDrawQuery, LotteryGame,
};

#[derive(Debug)]
enum QueryOutcome {
    Data(BTreeMap<String, Vec<i32>>),
    Empty,
    Rejected,
}

fn outcome_label(outcome: &QueryOutcome) -> String {
    match outcome {
        QueryOutcome::Data(values) => format!("data(count={})", values.len()),
        QueryOutcome::Empty => "empty".to_string(),
        QueryOutcome::Rejected => "rejected".to_string(),
    }
}

fn default_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn parse_yyyy_mm(value: &str) -> (i32, u8) {
    let mut parts = value.split('-');
    let year = parts
        .next()
        .expect("missing year")
        .parse::<i32>()
        .expect("invalid year");
    let month = parts
        .next()
        .expect("missing month")
        .parse::<u8>()
        .expect("invalid month");
    (year, month)
}

fn utc_current_yyyy_mm() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!("{:04}-{:02}", now.year(), u8::from(now.month()))
}

fn shift_month(year: i32, month: u8, delta: i32) -> (i32, u8) {
    let mut y = year;
    let mut m = i32::from(month);
    let mut d = delta;

    while d > 0 {
        if m == 12 {
            y += 1;
            m = 1;
        } else {
            m += 1;
        }
        d -= 1;
    }

    while d < 0 {
        if m == 1 {
            y -= 1;
            m = 12;
        } else {
            m -= 1;
        }
        d += 1;
    }

    (y, m as u8)
}

fn canonicalize(page: &HistoryDrawPage) -> BTreeMap<String, Vec<i32>> {
    let mut out = BTreeMap::new();
    for item in &page.items {
        let normalized = item
            .numbers
            .sorted
            .clone()
            .unwrap_or_else(|| item.numbers.base.numbers.clone());
        out.insert(item.period.clone(), normalized);
    }
    out
}

fn execute_query(
    source: &str,
    result: Result<HistoryDrawPage, DownloadError>,
) -> Result<QueryOutcome, String> {
    match result {
        Ok(page) => {
            if page.total_size == 0 && page.items.is_empty() {
                Ok(QueryOutcome::Empty)
            } else {
                Ok(QueryOutcome::Data(canonicalize(&page)))
            }
        }
        Err(err) => {
            let text = format!("{err}").to_ascii_lowercase();
            if text.contains("outside supported range")
                || text.contains("end_month must be greater than or equal")
            {
                Ok(QueryOutcome::Rejected)
            } else {
                Err(format!("{source} query failed: {err}"))
            }
        }
    }
}

fn compare_one_month(data_dir: &PathBuf, game: LotteryGame, month: &str) -> Result<(), String> {
    let query = HistoryDrawQuery::by_month(month.to_string());
    let local = execute_query("local", query_history_draw(data_dir, game, query.clone()))?;
    let remote = execute_query(
        "remote",
        query_history_draw_from_taiwan_lottery(game, query),
    )?;

    match (local, remote) {
        (QueryOutcome::Data(lhs), QueryOutcome::Data(rhs)) => {
            if lhs == rhs {
                Ok(())
            } else {
                let periods: BTreeSet<String> = lhs.keys().chain(rhs.keys()).cloned().collect();
                for period in periods {
                    let left = lhs.get(&period);
                    let right = rhs.get(&period);
                    if left != right {
                        return Err(format!(
                            "in-range mismatch game={} month={} period={} local={left:?} remote={right:?}",
                            game.metadata().display_name,
                            month,
                            period
                        ));
                    }
                }

                Err(format!(
                    "in-range mismatch game={} month={} local_count={} remote_count={}",
                    game.metadata().display_name,
                    month,
                    lhs.len(),
                    rhs.len()
                ))
            }
        }
        (QueryOutcome::Empty, QueryOutcome::Empty) => Ok(()),
        (QueryOutcome::Rejected, QueryOutcome::Rejected) => Ok(()),
        (left, right) => Err(format!(
            "in-range outcome mismatch game={} month={} local={} remote={}",
            game.metadata().display_name,
            month,
            outcome_label(&left),
            outcome_label(&right)
        )),
    }
}

// ===== GRANULAR PARITY TESTS =====

#[test]
#[ignore = "network-dependent granular boundary check"]
fn discontinued_game_boundary_before_start() {
    let data_dir = default_output_dir();
    let games = [
        (LotteryGame::Lotto38M6, "2006-06"),
        (LotteryGame::Lotto638, "2006-06"),
        (LotteryGame::TicTacToe, "2008-12"),
        (LotteryGame::Lotto1224, "2018-03"),
        (LotteryGame::Lotto740, "2015-04"),
    ];

    let mut failures = Vec::new();
    for (game, month_before_start) in &games {
        if let Err(err) = compare_one_month(&data_dir, *game, month_before_start) {
            failures.push(format!(
                "{}: {} (expected: empty)",
                game.metadata().display_name,
                err
            ));
        }
    }

    if !failures.is_empty() {
        for entry in &failures {
            println!("{entry}");
        }
    }
    assert!(failures.is_empty(), "boundary before start check failed");
}

#[test]
#[ignore = "network-dependent granular start month check"]
fn discontinued_game_start_month_matches() {
    let data_dir = default_output_dir();
    let games = [
        (LotteryGame::Lotto38M6, "2007-07"),
        (LotteryGame::Lotto638, "2007-07"),
        (LotteryGame::TicTacToe, "2009-02"),
        (LotteryGame::Lotto1224, "2018-04"),
        (LotteryGame::Lotto740, "2015-06"),
    ];

    let mut failures = Vec::new();
    for (game, start_month) in &games {
        if let Err(err) = compare_one_month(&data_dir, *game, start_month) {
            failures.push(format!("{}: {}", game.metadata().display_name, err));
        }
    }

    if !failures.is_empty() {
        for entry in &failures {
            println!("{entry}");
        }
    }
    assert!(failures.is_empty(), "start month parity check failed");
}

#[test]
#[ignore = "network-dependent granular end month check"]
fn discontinued_game_end_month_matches() {
    let data_dir = default_output_dir();
    let games = [
        (LotteryGame::Lotto38M6, "2023-12"),
        (LotteryGame::TicTacToe, "2013-12"),
        (LotteryGame::Lotto1224, "2023-12"),
        (LotteryGame::Lotto740, "2019-12"),
    ];

    let mut failures = Vec::new();
    for (game, end_month) in &games {
        if let Err(err) = compare_one_month(&data_dir, *game, end_month) {
            failures.push(format!("{}: {}", game.metadata().display_name, err));
        }
    }

    if !failures.is_empty() {
        for entry in &failures {
            println!("{entry}");
        }
    }
    assert!(failures.is_empty(), "end month parity check failed");
}

#[test]
#[ignore = "network-dependent granular boundary check"]
fn discontinued_game_boundary_after_end() {
    let data_dir = default_output_dir();
    let games = [
        (LotteryGame::Lotto38M6, "2024-01"),
        (LotteryGame::TicTacToe, "2014-01"),
        (LotteryGame::Lotto1224, "2024-01"),
        (LotteryGame::Lotto740, "2020-01"),
    ];

    let mut failures = Vec::new();
    for (game, month_after_end) in &games {
        if let Err(err) = compare_one_month(&data_dir, *game, month_after_end) {
            failures.push(format!(
                "{}: {} (expected: empty)",
                game.metadata().display_name,
                err
            ));
        }
    }

    if !failures.is_empty() {
        for entry in &failures {
            println!("{entry}");
        }
    }
    assert!(failures.is_empty(), "boundary after end check failed");
}

#[test]
#[ignore = "network-dependent granular boundary check"]
fn active_game_boundary_before_start() {
    let data_dir = default_output_dir();
    let games = [
        (LotteryGame::SuperLotto638, "2007-12"),
        (LotteryGame::Lotto649, "2006-12"),
        (LotteryGame::Daily539, "2006-12"),
        (LotteryGame::Lotto3D, "2006-12"),
        (LotteryGame::Lotto4D, "2006-12"),
        (LotteryGame::Lotto49M6, "2006-12"),
        (LotteryGame::Lotto39M5, "2009-12"),
    ];

    let mut failures = Vec::new();
    for (game, month_before_start) in &games {
        if let Err(err) = compare_one_month(&data_dir, *game, month_before_start) {
            failures.push(format!(
                "{}: {} (expected: empty)",
                game.metadata().display_name,
                err
            ));
        }
    }

    if !failures.is_empty() {
        for entry in &failures {
            println!("{entry}");
        }
    }
    assert!(
        failures.is_empty(),
        "active game boundary before start check failed"
    );
}

#[test]
#[ignore = "network-dependent granular start month check"]
fn active_game_start_month_matches() {
    let data_dir = default_output_dir();
    let games = [
        (LotteryGame::SuperLotto638, "2008-01"),
        (LotteryGame::Lotto649, "2007-01"),
        (LotteryGame::Daily539, "2007-01"),
        (LotteryGame::Lotto3D, "2007-01"),
        (LotteryGame::Lotto4D, "2007-01"),
        (LotteryGame::Lotto49M6, "2007-01"),
        (LotteryGame::Lotto39M5, "2010-01"),
    ];

    let mut failures = Vec::new();
    for (game, start_month) in &games {
        if let Err(err) = compare_one_month(&data_dir, *game, start_month) {
            failures.push(format!("{}: {}", game.metadata().display_name, err));
        }
    }

    if !failures.is_empty() {
        for entry in &failures {
            println!("{entry}");
        }
    }
    assert!(
        failures.is_empty(),
        "active game start month parity check failed"
    );
}

#[test]
#[ignore = "network-dependent granular recent data check"]
fn active_game_two_months_ago_matches() {
    let data_dir = default_output_dir();
    let games = [
        LotteryGame::SuperLotto638,
        LotteryGame::Lotto649,
        LotteryGame::Daily539,
        LotteryGame::Lotto3D,
        LotteryGame::Lotto4D,
        LotteryGame::Lotto49M6,
        LotteryGame::Lotto39M5,
    ];

    let now = utc_current_yyyy_mm();
    let (year, month) = parse_yyyy_mm(&now);
    let (two_months_ago_y, two_months_ago_m) = shift_month(year, month, -2);
    let two_months_ago = format!("{two_months_ago_y:04}-{two_months_ago_m:02}");

    let mut failures = Vec::new();
    for game in &games {
        if let Err(err) = compare_one_month(&data_dir, *game, &two_months_ago) {
            failures.push(format!("{}: {}", game.metadata().display_name, err));
        }
    }

    if !failures.is_empty() {
        for entry in &failures {
            println!("{entry}");
        }
    }
    assert!(
        failures.is_empty(),
        "active game two months ago parity check failed"
    );
}

fn compare_one_date(data_dir: &PathBuf, game: LotteryGame, date: &str) -> Result<(), String> {
    let query = HistoryDrawQuery::by_open_date(date.to_string());
    let local = execute_query("local", query_history_draw(data_dir, game, query.clone()))?;
    let remote = execute_query(
        "remote",
        query_history_draw_from_taiwan_lottery(game, query),
    )?;

    match (local, remote) {
        (QueryOutcome::Data(lhs), QueryOutcome::Data(rhs)) => {
            if lhs == rhs {
                Ok(())
            } else {
                let periods: BTreeSet<String> = lhs.keys().chain(rhs.keys()).cloned().collect();
                for period in periods {
                    let left = lhs.get(&period);
                    let right = rhs.get(&period);
                    if left != right {
                        return Err(format!(
                            "in-range mismatch game={} date={} period={} local={left:?} remote={right:?}",
                            game.metadata().display_name,
                            date,
                            period
                        ));
                    }
                }

                Err(format!(
                    "in-range mismatch game={} date={} local_count={} remote_count={}",
                    game.metadata().display_name,
                    date,
                    lhs.len(),
                    rhs.len()
                ))
            }
        }
        (QueryOutcome::Empty, QueryOutcome::Empty) => Ok(()),
        (QueryOutcome::Rejected, QueryOutcome::Rejected) => Ok(()),
        (left, right) => Err(format!(
            "in-range outcome mismatch game={} date={} local={} remote={}",
            game.metadata().display_name,
            date,
            outcome_label(&left),
            outcome_label(&right)
        )),
    }
}

#[test]
#[ignore = "network-dependent BingoBingo day-level parity check"]
fn bingo_bingo_boundary_before_local_start() {
    let data_dir = default_output_dir();
    let date_before_start = "2008-04-29";

    let mut failures = Vec::new();
    if let Err(err) = compare_one_date(&data_dir, LotteryGame::BingoBingo, date_before_start) {
        failures.push(format!(
            "BingoBingo: {} (expected: empty for local, may differ for remote)",
            err
        ));
    }

    if !failures.is_empty() {
        for entry in &failures {
            println!("{entry}");
        }
    }
    assert!(
        failures.is_empty(),
        "BingoBingo boundary before local start check failed"
    );
}

#[test]
#[ignore = "network-dependent BingoBingo day-level parity check"]
fn bingo_bingo_local_start_date_matches() {
    let data_dir = default_output_dir();
    // Local range starts 2008-04-30, Remote range starts 2024-01-01
    // Only test local start date
    let local_start = "2008-04-30";

    let mut failures = Vec::new();
    if let Err(err) = compare_one_date(&data_dir, LotteryGame::BingoBingo, local_start) {
        failures.push(format!("BingoBingo local start: {}", err));
    }

    if !failures.is_empty() {
        for entry in &failures {
            println!("{entry}");
        }
    }
    assert!(
        failures.is_empty(),
        "BingoBingo local start date parity check failed"
    );
}

#[test]
#[ignore = "network-dependent BingoBingo day-level parity check"]
fn bingo_bingo_remote_start_date_matches() {
    let data_dir = default_output_dir();
    // Remote range starts 2024-01-01 only
    let remote_start = "2024-01-01";

    let mut failures = Vec::new();
    if let Err(err) = compare_one_date(&data_dir, LotteryGame::BingoBingo, remote_start) {
        failures.push(format!("BingoBingo remote start: {}", err));
    }

    if !failures.is_empty() {
        for entry in &failures {
            println!("{entry}");
        }
    }
    assert!(
        failures.is_empty(),
        "BingoBingo remote start date parity check failed"
    );
}
