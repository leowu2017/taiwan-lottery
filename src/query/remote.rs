use std::collections::HashSet;

use crate::rule::{validate_query_range_for_game, YearMonth};
use crate::{
    DownloadError, HistoryDrawItem, HistoryDrawPage, HistoryDrawQuery, LotteryGame,
    SortedDrawNumbers,
};

const TAIWAN_LOTTERY_API_BASE_URL: &str = "https://api.taiwanlottery.com/TLCAPIWeB";

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

fn build_http_client() -> Result<reqwest::blocking::Client, DownloadError> {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(DownloadError::from)
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
        return Err(std::io::Error::other("history response content is not an object").into());
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
        .ok_or_else(|| std::io::Error::other("history response does not include draw records"))?;

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
    let page_size = 200usize;
    let mut page_num = 1usize;
    let mut total_size = 0usize;
    let mut all_items = Vec::new();

    loop {
        let response_body = client
            .get(url)
            .query(&[
                ("period", period),
                ("month", month),
                ("endMonth", end_month),
                ("pageNum", &page_num.to_string()),
                ("pageSize", &page_size.to_string()),
            ])
            .send()?
            .error_for_status()?
            .text()?;

        let response: TaiwanLotteryHistoryResponse = serde_json::from_str(&response_body)?;
        if response.rt_code != 0 {
            let message = response
                .rt_msg
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unknown history API error");
            return Err(std::io::Error::other(format!(
                "Taiwan Lottery history API returned rtCode={}, msg={message}",
                response.rt_code
            ))
            .into());
        }

        let content = response.content.as_ref().ok_or_else(|| {
            std::io::Error::other("Taiwan Lottery history API returned empty content")
        })?;
        let page = parse_history_draw_page(content)?;

        if page_num == 1 {
            total_size = page.total_size;
        }

        if page.items.is_empty() {
            break;
        }

        let fetched = page.items.len();
        all_items.extend(page.items);

        if fetched < page_size {
            break;
        }
        if total_size > 0 && all_items.len() >= total_size {
            break;
        }

        page_num += 1;
    }

    Ok(all_items)
}

fn parse_number_strings(values: Option<&[String]>) -> Vec<i32> {
    values
        .unwrap_or(&[])
        .iter()
        .filter_map(|value| value.trim().parse::<i32>().ok())
        .collect()
}

fn days_in_month(year: i32, month: u8) -> u8 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
            if leap {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn month_range_days(start: YearMonth, end: YearMonth) -> Vec<String> {
    let mut result = Vec::new();
    let mut cursor = start;

    while cursor <= end {
        let dim = days_in_month(cursor.year, cursor.month);
        for day in 1..=dim {
            result.push(format!("{:04}-{:02}-{:02}", cursor.year, cursor.month, day));
        }

        if cursor.month == 12 {
            cursor = YearMonth::new(cursor.year + 1, 1);
        } else {
            cursor = YearMonth::new(cursor.year, cursor.month + 1);
        }
    }

    result
}

fn fetch_bingo_result_by_open_date(
    client: &reqwest::blocking::Client,
    open_date: &str,
) -> Result<Vec<HistoryDrawItem>, DownloadError> {
    let page_size = 200usize;
    let mut page_num = 1usize;
    let mut all_items = Vec::new();

    loop {
        let response_body = client
            .get(format!("{TAIWAN_LOTTERY_API_BASE_URL}/Lottery/BingoResult"))
            .query(&[
                ("openDate", open_date),
                ("pageNum", &page_num.to_string()),
                ("pageSize", &page_size.to_string()),
            ])
            .send()?
            .error_for_status()?
            .text()?;

        let response: TaiwanLotteryBingoResponse = serde_json::from_str(&response_body)?;
        if response.rt_code != 0 {
            let message = response
                .rt_msg
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("unknown bingo API error");
            return Err(std::io::Error::other(format!(
                "Taiwan Lottery bingo API returned rtCode={}, msg={message}",
                response.rt_code
            ))
            .into());
        }

        let content = response.content.ok_or_else(|| {
            std::io::Error::other("Taiwan Lottery bingo API returned empty content")
        })?;
        let _total_size = content.total_size;
        if content.bingo_query_result.is_empty() {
            break;
        }

        let fetched = content.bingo_query_result.len();
        for record in content.bingo_query_result {
            let period = record
                .draw_term
                .map(|value| value.to_string())
                .unwrap_or_default();
            if period.is_empty() {
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
        }

        if fetched < page_size {
            break;
        }
        page_num += 1;
    }

    Ok(all_items)
}

fn query_bingo_history_with_client(
    client: &reqwest::blocking::Client,
    query: &HistoryDrawQuery,
) -> Result<HistoryDrawPage, DownloadError> {
    let (period, month, end_month) = query.normalized_params()?;
    if !period.is_empty() {
        return Err(std::io::Error::other(
            "remote period query is not supported for BINGO BINGO; use month or month-range",
        )
        .into());
    }

    let start = YearMonth::parse_yyyy_mm(month)?;
    let end = YearMonth::parse_yyyy_mm(end_month)?;
    if end < start {
        return Err(
            std::io::Error::other("end_month must be greater than or equal to month").into(),
        );
    }

    let mut all_items = Vec::new();
    for open_date in month_range_days(start, end) {
        let mut items = fetch_bingo_result_by_open_date(client, &open_date)?;
        all_items.append(&mut items);
    }

    all_items.sort_by(|a, b| a.period.cmp(&b.period));
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
    all_items.sort_by(|a, b| a.period.cmp(&b.period));

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

    #[test]
    fn days_in_month_handles_leap_and_common_years() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2025, 2), 28);
        assert_eq!(days_in_month(2026, 6), 30);
    }

    #[test]
    fn month_range_days_spans_multiple_months() {
        let days = month_range_days(YearMonth::new(2026, 6), YearMonth::new(2026, 7));
        assert_eq!(days.first().map(String::as_str), Some("2026-06-01"));
        assert_eq!(days.last().map(String::as_str), Some("2026-07-31"));
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
}
