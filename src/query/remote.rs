use std::collections::HashSet;

use super::common::{period_newest_first, validate_query_range_for_game};
use crate::http::{build_http_client, polite_pause, send_ok};
use crate::{
    DownloadError, HistoryDrawItem, HistoryDrawPage, HistoryDrawQuery, LotteryGame,
    RemoteQueryParamSupport, SortedDrawNumbers,
};

const TAIWAN_LOTTERY_API_BASE_URL: &str = "https://api.taiwanlottery.com/TLCAPIWeB";
/// Page cap (at 200 items per page) so a misbehaving server cannot make a query run forever.
const MAX_PAGES: usize = 500;

pub(crate) const fn remote_query_param_support(game: LotteryGame) -> RemoteQueryParamSupport {
    match game {
        LotteryGame::BingoBingo => RemoteQueryParamSupport {
            month: false,
            end_month: false,
            open_date: true,
            period: true,
        },
        _ => RemoteQueryParamSupport {
            month: true,
            end_month: true,
            open_date: false,
            period: true,
        },
    }
}

#[derive(Debug, serde::Deserialize)]
struct TaiwanLotteryHistoryResponse {
    #[serde(rename = "rtCode")]
    rt_code: i32,
    #[serde(rename = "rtMsg")]
    rt_msg: Option<String>,
    content: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
struct TaiwanLotteryBingoResponse {
    #[serde(rename = "rtCode")]
    rt_code: i32,
    #[serde(rename = "rtMsg")]
    rt_msg: Option<String>,
    content: Option<TaiwanLotteryBingoContent>,
}

#[derive(Debug, serde::Deserialize)]
struct TaiwanLotteryBingoContent {
    #[serde(rename = "totalSize")]
    total_size: Option<u64>,
    #[serde(rename = "bingoQueryResult")]
    bingo_query_result: Vec<TaiwanLotteryBingoItem>,
}

#[derive(Debug, serde::Deserialize)]
struct TaiwanLotteryBingoItem {
    #[serde(rename = "drawTerm")]
    draw_term: Option<u64>,
    #[serde(rename = "dDate")]
    draw_date: Option<String>,
    #[serde(rename = "bigShowOrder")]
    big_show_order: Option<Vec<String>>,
    #[serde(rename = "openShowOrder")]
    open_show_order: Option<Vec<String>>,
}

fn json_value_to_i32_vec(value: Option<&serde_json::Value>) -> Vec<i32> {
    let Some(serde_json::Value::Array(values)) = value else {
        return Vec::new();
    };

    values
        .iter()
        .filter_map(|entry| entry.as_i64())
        .filter_map(|entry| i32::try_from(entry).ok())
        .collect()
}

pub(crate) fn parse_history_draw_page(
    content: &serde_json::Value,
) -> Result<HistoryDrawPage, DownloadError> {
    let serde_json::Value::Object(content_obj) = content else {
        return Err(DownloadError::data(
            "history response content is not an object",
        ));
    };

    let total_size = content_obj
        .get("totalSize")
        .and_then(|value| value.as_u64())
        .and_then(|value| usize::try_from(value).ok())
        .unwrap_or(0);

    if total_size == 0 {
        let has_result_array = content_obj.iter().any(|(key, value)| {
            key.ends_with("Res") && matches!(value, serde_json::Value::Array(_))
        });
        if has_result_array {
            return Ok(HistoryDrawPage {
                total_size: 0,
                items: Vec::new(),
            });
        }
    }

    let records = content_obj
        .values()
        .find_map(|value| {
            let serde_json::Value::Array(records) = value else {
                return None;
            };

            let has_draw_fields = records.iter().any(|record| {
                let serde_json::Value::Object(record_obj) = record else {
                    return false;
                };
                record_obj.contains_key("drawNumberAppear")
                    || record_obj.contains_key("drawNumberSize")
            });

            if has_draw_fields {
                Some(records)
            } else {
                None
            }
        })
        .ok_or_else(|| DownloadError::data("history response does not include draw records"))?;

    let mut items = Vec::new();
    for record in records {
        let serde_json::Value::Object(record_obj) = record else {
            continue;
        };

        let numbers_sorted = json_value_to_i32_vec(record_obj.get("drawNumberSize"));
        let numbers_draw = json_value_to_i32_vec(record_obj.get("drawNumberAppear"));
        if numbers_sorted.is_empty() && numbers_draw.is_empty() {
            continue;
        }

        let period = match record_obj.get("period") {
            Some(serde_json::Value::String(value)) => value.clone(),
            Some(serde_json::Value::Number(value)) => value.to_string(),
            _ => String::new(),
        };

        let date = record_obj
            .get("lotteryDate")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned);
        let redeemable_date = record_obj
            .get("redeemableDate")
            .and_then(|value| value.as_str())
            .map(ToOwned::to_owned);

        let sorted_numbers = (!numbers_sorted.is_empty()).then_some(numbers_sorted);
        let base_numbers = if numbers_draw.is_empty() {
            sorted_numbers.clone().unwrap_or_default()
        } else {
            numbers_draw
        };

        items.push(HistoryDrawItem {
            period,
            date,
            redeemable_date,
            numbers: SortedDrawNumbers::new(base_numbers, sorted_numbers),
        });
    }

    Ok(HistoryDrawPage { total_size, items })
}

fn fetch_all_pages_from_url(
    client: &reqwest::blocking::Client,
    url: &str,
    period: &str,
    month: &str,
    end_month: &str,
) -> Result<Vec<HistoryDrawItem>, DownloadError> {
    fetch_all_pages_with_limit(client, url, period, month, end_month, MAX_PAGES)
}

fn page_limit_error(api: &str) -> DownloadError {
    DownloadError::data(format!(
        "Taiwan Lottery {api} API kept returning full pages; giving up to avoid an endless loop"
    ))
}

fn fetch_all_pages_with_limit(
    client: &reqwest::blocking::Client,
    url: &str,
    period: &str,
    month: &str,
    end_month: &str,
    max_pages: usize,
) -> Result<Vec<HistoryDrawItem>, DownloadError> {
    let page_size = 200usize;
    let mut page_num = 1usize;
    let mut total_size = 0usize;
    let mut all_items = Vec::new();
    let mut seen = HashSet::new();

    loop {
        let page_num_text = page_num.to_string();
        let page_size_text = page_size.to_string();
        let response_body = send_ok(|| {
            client.get(url).query(&[
                ("period", period),
                ("month", month),
                ("endMonth", end_month),
                ("pageNum", page_num_text.as_str()),
                ("pageSize", page_size_text.as_str()),
            ])
        })?
        .text()?;

        let response: TaiwanLotteryHistoryResponse = serde_json::from_str(&response_body)?;
        if response.rt_code != 0 {
            let message = response
                .rt_msg
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unknown history API error");
            return Err(DownloadError::data(format!(
                "Taiwan Lottery history API returned rtCode={}, msg={message}",
                response.rt_code
            )));
        }

        let content = response.content.as_ref().ok_or_else(|| {
            DownloadError::data("Taiwan Lottery history API returned empty content")
        })?;
        let page = parse_history_draw_page(content)?;

        if page_num == 1 {
            total_size = page.total_size;
        }

        if page.items.is_empty() {
            break;
        }

        let fetched = page.items.len();
        let fresh: Vec<HistoryDrawItem> = page
            .items
            .into_iter()
            .filter(|item| seen.insert(item.period.clone()))
            .collect();
        // A page without new periods means the server ignores pageNum; stop instead of looping.
        if fresh.is_empty() {
            break;
        }
        all_items.extend(fresh);

        if fetched < page_size {
            break;
        }
        if total_size > 0 && all_items.len() >= total_size {
            break;
        }
        if page_num >= max_pages {
            return Err(page_limit_error("history"));
        }

        polite_pause();
        page_num += 1;
    }

    Ok(all_items)
}

fn parse_number_strings(values: Option<&[String]>) -> Vec<i32> {
    // Bingo responses encode numbers as strings rather than numeric JSON values.
    values
        .unwrap_or(&[])
        .iter()
        .filter_map(|value| value.trim().parse::<i32>().ok())
        .collect()
}

fn fetch_bingo_result_by_open_date(
    client: &reqwest::blocking::Client,
    open_date: &str,
) -> Result<Vec<HistoryDrawItem>, DownloadError> {
    fetch_bingo_result_by_filter(client, "openDate", open_date)
}

fn fetch_bingo_result_by_period(
    client: &reqwest::blocking::Client,
    period: &str,
) -> Result<Vec<HistoryDrawItem>, DownloadError> {
    fetch_bingo_result_by_filter(client, "period", period)
}

fn fetch_bingo_result_by_filter(
    client: &reqwest::blocking::Client,
    key: &str,
    value: &str,
) -> Result<Vec<HistoryDrawItem>, DownloadError> {
    fetch_bingo_results_with_limit(
        client,
        &format!("{TAIWAN_LOTTERY_API_BASE_URL}/Lottery/BingoResult"),
        key,
        value,
        MAX_PAGES,
    )
}

fn fetch_bingo_results_with_limit(
    client: &reqwest::blocking::Client,
    url: &str,
    key: &str,
    value: &str,
    max_pages: usize,
) -> Result<Vec<HistoryDrawItem>, DownloadError> {
    let page_size = 200usize;
    let mut page_num = 1usize;
    let mut all_items = Vec::new();
    let mut seen = HashSet::new();

    loop {
        let page_num_text = page_num.to_string();
        let page_size_text = page_size.to_string();
        let response_body = send_ok(|| {
            client.get(url).query(&[
                (key, value),
                ("pageNum", page_num_text.as_str()),
                ("pageSize", page_size_text.as_str()),
            ])
        })?
        .text()?;

        let response: TaiwanLotteryBingoResponse = serde_json::from_str(&response_body)?;
        if response.rt_code != 0 {
            let message = response
                .rt_msg
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unknown bingo API error");
            return Err(DownloadError::data(format!(
                "Taiwan Lottery bingo API returned rtCode={}, msg={message}",
                response.rt_code
            )));
        }

        let content = response.content.ok_or_else(|| {
            DownloadError::data("Taiwan Lottery bingo API returned empty content")
        })?;
        let _total_size = content.total_size;
        if content.bingo_query_result.is_empty() {
            break;
        }

        let fetched = content.bingo_query_result.len();
        let mut added = false;
        for record in content.bingo_query_result {
            let period = record
                .draw_term
                .map(|value| value.to_string())
                .unwrap_or_default();
            if period.is_empty() || !seen.insert(period.clone()) {
                continue;
            }

            let numbers_draw = parse_number_strings(record.open_show_order.as_deref());
            let numbers_sorted = parse_number_strings(record.big_show_order.as_deref());
            if numbers_draw.is_empty() && numbers_sorted.is_empty() {
                continue;
            }

            let sorted_numbers = (!numbers_sorted.is_empty()).then_some(numbers_sorted);
            let base_numbers = if numbers_draw.is_empty() {
                sorted_numbers.clone().unwrap_or_default()
            } else {
                numbers_draw
            };

            all_items.push(HistoryDrawItem {
                period,
                date: record.draw_date,
                redeemable_date: None,
                numbers: SortedDrawNumbers::new(base_numbers, sorted_numbers),
            });
            added = true;
        }

        // A page without new periods means the server ignores pageNum; stop instead of looping.
        if !added {
            break;
        }

        if fetched < page_size {
            break;
        }
        if page_num >= max_pages {
            return Err(page_limit_error("bingo"));
        }
        polite_pause();
        page_num += 1;
    }

    Ok(all_items)
}

fn query_bingo_history_with_client(
    client: &reqwest::blocking::Client,
    query: &HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    // Bingo uses openDate/period rather than month/endMonth.
    let period = query.period.as_deref().unwrap_or("").trim();
    if !period.is_empty() {
        let mut items = fetch_bingo_result_by_period(client, period)?;
        items.sort_by(|a, b| period_newest_first(&a.period, &b.period));
        let total_size = items.len();
        return Ok(HistoryDrawPage { total_size, items });
    }

    let open_date = query.open_date.as_deref().unwrap_or("").trim();
    if open_date.is_empty() {
        return Err(DownloadError::invalid_query(
            "open_date or period is required for BINGO BINGO queries",
        ));
    }

    let mut all_items = fetch_bingo_result_by_open_date(client, open_date)?;

    all_items.sort_by(|a, b| period_newest_first(&a.period, &b.period));
    all_items.dedup_by(|a, b| a.period == b.period);

    let total_size = all_items.len();
    Ok(HistoryDrawPage {
        total_size,
        items: all_items,
    })
}

pub(crate) fn query_history_draw_with_client(
    client: &reqwest::blocking::Client,
    game: LotteryGame,
    query: &HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    validate_query_range_for_game(game, query)?;

    if game == LotteryGame::BingoBingo {
        return query_bingo_history_with_client(client, query);
    }

    let (period, month, end_month) = query.normalized_params()?;
    let main_url = format!("{TAIWAN_LOTTERY_API_BASE_URL}{}", game.path());
    let mut all_items = fetch_all_pages_from_url(client, &main_url, period, month, end_month)?;

    if let Some(history_path) = game.history_session_path() {
        let history_url = format!("{TAIWAN_LOTTERY_API_BASE_URL}{history_path}");
        let history_items =
            fetch_all_pages_from_url(client, &history_url, period, month, end_month)?;
        all_items.extend(history_items);
    }

    let mut seen = HashSet::new();
    all_items.retain(|item| seen.insert(item.period.clone()));
    all_items.sort_by(|a, b| period_newest_first(&a.period, &b.period));

    let total_size = all_items.len();
    Ok(HistoryDrawPage {
        total_size,
        items: all_items,
    })
}

pub(crate) fn query_history_draw_from_taiwan_lottery(
    game: LotteryGame,
    query: &HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    let client = build_http_client()?;
    query_history_draw_with_client(&client, game, query)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::common::{
        current_utc_year_month, days_in_month, game_query_month_bounds,
        parse_open_date_to_year_month, YearMonth,
    };
    use crate::HistoryDrawQuery;

    #[test]
    fn days_in_month_handles_leap_and_common_years() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2025, 2), 28);
        assert_eq!(days_in_month(2026, 6), 30);
    }

    #[test]
    fn parse_open_date_to_year_month_accepts_valid_date() {
        let month = parse_open_date_to_year_month("2026-07-08").expect("must parse open_date");
        assert_eq!(month, YearMonth::new(2026, 7));
    }

    #[test]
    fn parse_open_date_to_year_month_rejects_invalid_date() {
        let err = parse_open_date_to_year_month("2026-02-30")
            .expect_err("invalid day should be rejected");
        assert!(matches!(err, DownloadError::InvalidQuery(_)));
    }

    #[test]
    fn parse_history_draw_page_accepts_empty_result_array() {
        let sample = serde_json::json!({
            "totalSize": 0,
            "lotto638Res": []
        });

        let page = parse_history_draw_page(&sample).expect("must parse empty remote page");
        assert_eq!(page.total_size, 0);
        assert!(page.items.is_empty());
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

        let page = parse_history_draw_page(&sample).expect("parse history draw page");
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
    fn validate_query_range_rejects_out_of_term_month_for_lotto_1224() {
        let query = HistoryDrawQuery::by_month("2013-12");
        let err = validate_query_range_for_game(LotteryGame::Lotto1224, &query)
            .expect_err("1224 should not allow third-term month");
        assert!(matches!(err, DownloadError::InvalidQuery(_)));
    }

    #[test]
    fn validate_query_range_rejects_out_of_term_month_for_tic_tac_toe() {
        let query = HistoryDrawQuery::by_month("2014-01");
        let err = validate_query_range_for_game(LotteryGame::TicTacToe, &query)
            .expect_err("tic-tac-toe should not allow fourth-term month");
        assert!(matches!(err, DownloadError::InvalidQuery(_)));
    }

    #[test]
    fn validate_query_range_rejects_out_of_term_period_for_lotto_740() {
        let query = HistoryDrawQuery::by_period("113000001");
        let err = validate_query_range_for_game(LotteryGame::Lotto740, &query)
            .expect_err("740 should not allow fifth-term period");
        assert!(matches!(err, DownloadError::InvalidQuery(_)));
    }

    #[test]
    fn validate_query_range_accepts_third_to_fourth_overlap_game() {
        let query = HistoryDrawQuery::by_month("2023-12");
        validate_query_range_for_game(LotteryGame::Lotto38M6, &query)
            .expect("38M6 should allow fourth-term month");
    }

    #[test]
    fn validate_query_range_rejects_future_month_for_fifth_active_game() {
        let now = current_utc_year_month();
        let (future_year, future_month) = if now.month == 12 {
            (now.year + 1, 1)
        } else {
            (now.year, now.month + 1)
        };
        let query = HistoryDrawQuery::by_month(format!("{future_year:04}-{future_month:02}"));
        let err = validate_query_range_for_game(LotteryGame::Lotto649, &query)
            .expect_err("lotto649 should not allow future month");
        assert!(matches!(err, DownloadError::InvalidQuery(_)));
    }

    #[test]
    fn validate_query_range_rejects_month_query_for_bingo_bingo() {
        let query = HistoryDrawQuery::by_month("2026-07");
        let err = validate_query_range_for_game(LotteryGame::BingoBingo, &query)
            .expect_err("bingo should require open_date or period");
        assert!(matches!(err, DownloadError::InvalidQuery(_)));
    }

    #[test]
    fn validate_query_range_accepts_open_date_query_for_bingo_bingo() {
        let query = HistoryDrawQuery::by_open_date("2026-07-08");
        validate_query_range_for_game(LotteryGame::BingoBingo, &query)
            .expect("bingo should allow open_date query");
    }

    #[test]
    fn validate_query_range_rejects_open_date_for_non_bingo_game() {
        let query = HistoryDrawQuery::by_open_date("2026-07-08");
        let err = validate_query_range_for_game(LotteryGame::Lotto649, &query)
            .expect_err("non-bingo should reject open_date");
        assert!(matches!(err, DownloadError::InvalidQuery(_)));
    }

    #[test]
    fn lottery_game_query_month_range_caps_active_game_to_current_month() {
        let (start, end) = game_query_month_bounds(LotteryGame::Lotto649);
        let now = current_utc_year_month();
        assert_eq!(start.to_yyyy_mm(), "2007-01");
        assert_eq!(
            end.to_yyyy_mm(),
            format!("{:04}-{:02}", now.year, now.month)
        );
    }

    #[test]
    fn lottery_game_query_month_range_for_bingo_bingo_uses_2024_start() {
        let (start, end) = game_query_month_bounds(LotteryGame::BingoBingo);
        let now = current_utc_year_month();
        assert_eq!(start.to_yyyy_mm(), "2024-01");
        assert_eq!(
            end.to_yyyy_mm(),
            format!("{:04}-{:02}", now.year, now.month)
        );
    }

    #[test]
    fn remote_query_param_support_matches_empirical_behavior() {
        let non_bingo = remote_query_param_support(LotteryGame::Lotto649);
        assert!(non_bingo.month);
        assert!(non_bingo.end_month);
        assert!(!non_bingo.open_date);
        assert!(non_bingo.period);

        let bingo = remote_query_param_support(LotteryGame::BingoBingo);
        assert!(!bingo.month);
        assert!(!bingo.end_month);
        assert!(bingo.open_date);
        assert!(bingo.period);
    }

    fn history_page_body(first_period: usize, count: usize) -> String {
        let records: Vec<String> = (first_period..first_period + count)
            .map(|period| {
                format!(
                    r#"{{"period":{period},"drawNumberSize":[1,2,3],"drawNumberAppear":[3,2,1]}}"#
                )
            })
            .collect();
        format!(
            r#"{{"rtCode":0,"content":{{"totalSize":100000,"lotto649Res":[{}]}}}}"#,
            records.join(",")
        )
    }

    fn bingo_page_body(first_term: usize, count: usize) -> String {
        let records: Vec<String> = (first_term..first_term + count)
            .map(|term| {
                format!(
                    r#"{{"drawTerm":{term},"dDate":"2026-01-01","bigShowOrder":["01","02"],"openShowOrder":["02","01"]}}"#
                )
            })
            .collect();
        format!(
            r#"{{"rtCode":0,"content":{{"totalSize":0,"bingoQueryResult":[{}]}}}}"#,
            records.join(",")
        )
    }

    fn page_number(target: &str) -> usize {
        target
            .split("pageNum=")
            .nth(1)
            .and_then(|value| value.split('&').next())
            .and_then(|value| value.parse().ok())
            .unwrap_or(1)
    }

    #[test]
    fn history_paging_stops_when_server_ignores_page_number() {
        use crate::test_support::{MockResponse, MockServer};

        let server = MockServer::start(|_| MockResponse::ok(history_page_body(1, 200)));
        let client = build_http_client().expect("client");

        let items = fetch_all_pages_from_url(&client, &server.base_url, "", "2026-01", "2026-01")
            .expect("must terminate");
        assert_eq!(items.len(), 200);
        assert_eq!(server.hits(), 2);
    }

    #[test]
    fn history_paging_fails_when_page_cap_is_exceeded() {
        use crate::test_support::{MockResponse, MockServer};

        let server = MockServer::start(|target| {
            MockResponse::ok(history_page_body(page_number(target) * 1000, 200))
        });
        let client = build_http_client().expect("client");

        let err =
            fetch_all_pages_with_limit(&client, &server.base_url, "", "2026-01", "2026-01", 3)
                .expect_err("must hit the cap");
        assert!(matches!(err, DownloadError::Data(_)));
        assert_eq!(server.hits(), 3);
    }

    #[test]
    fn bingo_paging_stops_when_server_ignores_page_number() {
        use crate::test_support::{MockResponse, MockServer};

        let server = MockServer::start(|_| MockResponse::ok(bingo_page_body(1, 200)));
        let client = build_http_client().expect("client");

        let items = fetch_bingo_results_with_limit(
            &client,
            &server.base_url,
            "openDate",
            "2026-01-01",
            MAX_PAGES,
        )
        .expect("must terminate");
        assert_eq!(items.len(), 200);
        assert_eq!(server.hits(), 2);
    }

    #[test]
    fn bingo_paging_fails_when_page_cap_is_exceeded() {
        use crate::test_support::{MockResponse, MockServer};

        let server = MockServer::start(|target| {
            MockResponse::ok(bingo_page_body(page_number(target) * 1000, 200))
        });
        let client = build_http_client().expect("client");

        let err =
            fetch_bingo_results_with_limit(&client, &server.base_url, "openDate", "2026-01-01", 3)
                .expect_err("must hit the cap");
        assert!(matches!(err, DownloadError::Data(_)));
    }
}
