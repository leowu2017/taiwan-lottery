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

fn prev_month(year: i32, month: u8) -> (i32, u8) {
    if month == 1 {
        (year - 1, 12)
    } else {
        (year, month - 1)
    }
}

fn next_month(year: i32, month: u8) -> (i32, u8) {
    if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    }
}

fn effective_end_boundary_months(range: &taiwan_lottery::LotteryGameQueryRange) -> Vec<String> {
    let current = utc_current_yyyy_mm();
    if range.max_month != current {
        return vec![range.max_month.clone()];
    }

    let (year, month) = parse_yyyy_mm(&range.max_month);
    let (prev1_y, prev1_m) = shift_month(year, month, -1);
    let (prev2_y, prev2_m) = shift_month(year, month, -2);
    vec![
        format!("{prev1_y:04}-{prev1_m:02}"),
        format!("{prev2_y:04}-{prev2_m:02}"),
    ]
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

fn check_out_of_range_months(data_dir: &PathBuf, game: LotteryGame) -> Result<(), String> {
    let range = game.query_month_range();
    let (min_year, min_month) = parse_yyyy_mm(&range.min_month);
    let (max_year, max_month) = parse_yyyy_mm(&range.max_month);

    let candidates = [
        prev_month(min_year, min_month),
        next_month(max_year, max_month),
    ];

    for (year, month) in candidates {
        let month_text = format!("{year:04}-{month:02}");
        let query = HistoryDrawQuery::by_month(month_text.clone());

        let local = execute_query("local", query_history_draw(data_dir, game, query.clone()))?;
        let remote = execute_query(
            "remote",
            query_history_draw_from_taiwan_lottery(game, query),
        )?;

        match (local, remote) {
            (QueryOutcome::Empty, QueryOutcome::Empty) => {}
            (QueryOutcome::Rejected, QueryOutcome::Rejected) => {}
            (left, right) => {
                return Err(format!(
                    "out-of-range mismatch game={} month={} local={} remote={}",
                    game.metadata().display_name,
                    month_text,
                    outcome_label(&left),
                    outcome_label(&right)
                ));
            }
        }
    }

    Ok(())
}

#[test]
#[ignore = "network-dependent parity check"]
fn local_remote_boundary_and_out_of_range_match() {
    let data_dir = default_output_dir();
    let mut checked_boundary_months: usize = 0;
    let mut boundary_failures = Vec::new();
    let mut out_of_range_failures = Vec::new();

    for game in LotteryGame::ALL {
        let range = game.query_month_range();
        let mut boundary_months = Vec::new();
        boundary_months.push(range.min_month.clone());
        boundary_months.extend(effective_end_boundary_months(&range));

        for month in &boundary_months {
            checked_boundary_months += 1;
            if let Err(err) = compare_one_month(&data_dir, game, month) {
                boundary_failures.push(err);
            }
        }

        if let Err(err) = check_out_of_range_months(&data_dir, game) {
            out_of_range_failures.push(err);
        }
    }

    println!("checked_boundary_months={checked_boundary_months}");
    println!("boundary_failures={}", boundary_failures.len());
    println!("out_of_range_failures={}", out_of_range_failures.len());

    if !boundary_failures.is_empty() {
        println!("---- boundary failure sample (up to 20) ----");
        for entry in boundary_failures.iter().take(20) {
            println!("{entry}");
        }
    }

    if !out_of_range_failures.is_empty() {
        println!("---- out-of-range failure sample (up to 20) ----");
        for entry in out_of_range_failures.iter().take(20) {
            println!("{entry}");
        }
    }

    assert!(
        boundary_failures.is_empty() && out_of_range_failures.is_empty(),
        "local/remote parity mismatch found"
    );
}
