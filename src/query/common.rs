use crate::rule::query_date_range_for_game;
use crate::{DownloadError, HistoryDrawQuery, LotteryGame};

/// Represents a year-month pair for date range queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct YearMonth {
    pub(crate) year: i32,
    pub(crate) month: u8,
}

impl YearMonth {
    pub(crate) const fn new(year: i32, month: u8) -> Self {
        Self { year, month }
    }

    pub(crate) fn parse_yyyy_mm(value: &str) -> Result<Self, DownloadError> {
        let trimmed = value.trim();
        let mut parts = trimmed.split('-');
        let year = parts
            .next()
            .ok_or_else(|| std::io::Error::other("month must be in YYYY-MM format"))?
            .parse::<i32>()
            .map_err(|_| std::io::Error::other("month year must be a valid number"))?;
        let month = parts
            .next()
            .ok_or_else(|| std::io::Error::other("month must be in YYYY-MM format"))?
            .parse::<u8>()
            .map_err(|_| std::io::Error::other("month must be a valid number"))?;

        if parts.next().is_some() {
            return Err(std::io::Error::other("month must be in YYYY-MM format").into());
        }
        if !(1..=12).contains(&month) {
            return Err(std::io::Error::other("month must be between 01 and 12").into());
        }

        Ok(Self::new(year, month))
    }

    pub(crate) fn to_yyyy_mm(self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }
}

pub(crate) fn current_utc_year_month() -> YearMonth {
    let now = time::OffsetDateTime::now_utc();
    YearMonth::new(now.year(), u8::from(now.month()))
}

pub(crate) fn parse_period_year(period: &str) -> Result<i32, DownloadError> {
    let trimmed = period.trim();
    if trimmed.len() < 3 {
        return Err(std::io::Error::other("period must include at least 3 ROC year digits").into());
    }

    let roc_year = trimmed
        .chars()
        .take(3)
        .collect::<String>()
        .parse::<i32>()
        .map_err(|_| std::io::Error::other("period must start with 3 ROC year digits"))?;
    Ok(roc_year + 1911)
}

pub(crate) fn parse_open_date_to_year_month(open_date: &str) -> Result<YearMonth, DownloadError> {
    let trimmed = open_date.trim();
    let mut parts = trimmed.split('-');

    let year = parts
        .next()
        .ok_or_else(|| std::io::Error::other("open_date must be in YYYY-MM-DD format"))?
        .parse::<i32>()
        .map_err(|_| std::io::Error::other("open_date year must be a valid number"))?;
    let month = parts
        .next()
        .ok_or_else(|| std::io::Error::other("open_date must be in YYYY-MM-DD format"))?
        .parse::<u8>()
        .map_err(|_| std::io::Error::other("open_date month must be a valid number"))?;
    let day = parts
        .next()
        .ok_or_else(|| std::io::Error::other("open_date must be in YYYY-MM-DD format"))?
        .parse::<u8>()
        .map_err(|_| std::io::Error::other("open_date day must be a valid number"))?;

    if parts.next().is_some() {
        return Err(std::io::Error::other("open_date must be in YYYY-MM-DD format").into());
    }
    if !(1..=12).contains(&month) {
        return Err(std::io::Error::other("open_date month must be between 01 and 12").into());
    }

    let max_day = days_in_month(year, month);
    if day == 0 || day > max_day {
        return Err(std::io::Error::other(format!(
            "open_date day must be between 01 and {max_day:02}"
        ))
        .into());
    }

    Ok(YearMonth::new(year, month))
}

pub(crate) fn parse_date_month(date: &str) -> Option<String> {
    let normalized = date.trim().replace('/', "-");
    if normalized.len() >= 7 {
        Some(normalized[..7].to_string())
    } else {
        None
    }
}

pub(crate) fn days_in_month(year: i32, month: u8) -> u8 {
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

pub(crate) fn game_query_month_bounds(game: LotteryGame) -> (YearMonth, YearMonth) {
    let date_range = query_date_range_for_game(game);
    let start = YearMonth::new(date_range.start_year, date_range.start_month);

    let end = match (date_range.end_year, date_range.end_month) {
        (Some(year), Some(month)) => {
            // Discontinued game: use the end date as-is
            YearMonth::new(year, month)
        }
        (None, None) => {
            // Active game: cap to current month (don't allow future queries)
            let now = current_utc_year_month();
            now
        }
        _ => unreachable!(),
    };

    (start, end)
}

fn ensure_period_year_in_range(
    game: LotteryGame,
    period: &str,
    query_year: i32,
    allowed_start: YearMonth,
    allowed_end: YearMonth,
) -> Result<(), DownloadError> {
    if query_year < allowed_start.year || query_year > allowed_end.year {
        return Err(std::io::Error::other(format!(
            "query period {period} (AD {query_year}) is outside supported range {}-{:02} to {}-{:02} for {}",
            allowed_start.year,
            allowed_start.month,
            allowed_end.year,
            allowed_end.month,
            game.metadata().display_name
        ))
        .into());
    }

    Ok(())
}

fn ensure_month_range_in_range(
    game: LotteryGame,
    month: &str,
    end_month: &str,
    query_start: YearMonth,
    query_end: YearMonth,
    allowed_start: YearMonth,
    allowed_end: YearMonth,
) -> Result<(), DownloadError> {
    if query_end < query_start {
        return Err(
            std::io::Error::other("end_month must be greater than or equal to month").into(),
        );
    }

    if query_start < allowed_start || query_end > allowed_end {
        return Err(std::io::Error::other(format!(
            "query month range {} to {} is outside supported range {}-{:02} to {}-{:02} for {}",
            month,
            end_month,
            allowed_start.year,
            allowed_start.month,
            allowed_end.year,
            allowed_end.month,
            game.metadata().display_name
        ))
        .into());
    }

    Ok(())
}

pub(crate) fn validate_query_range_for_game(
    game: LotteryGame,
    query: &HistoryDrawQuery,
) -> Result<(), DownloadError> {
    let (allowed_start, allowed_end) = game_query_month_bounds(game);
    let period = query.period.as_deref().unwrap_or("").trim();

    if game == LotteryGame::BingoBingo {
        if !period.is_empty() {
            let query_year = parse_period_year(period)?;
            ensure_period_year_in_range(game, period, query_year, allowed_start, allowed_end)?;
            return Ok(());
        }

        let open_date = query.open_date.as_deref().unwrap_or("").trim();
        if open_date.is_empty() {
            return Err(std::io::Error::other(
                "open_date or period is required for BINGO BINGO remote query",
            )
            .into());
        }

        let query_month = parse_open_date_to_year_month(open_date)?;
        if query_month < allowed_start || query_month > allowed_end {
            return Err(std::io::Error::other(format!(
                "query open_date {open_date} is outside supported range {}-{:02} to {}-{:02} for {}",
                allowed_start.year,
                allowed_start.month,
                allowed_end.year,
                allowed_end.month,
                game.metadata().display_name
            ))
            .into());
        }

        return Ok(());
    }

    if query
        .open_date
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
    {
        return Err(std::io::Error::other(
            "open_date is not supported for this game in remote query",
        )
        .into());
    }

    let (_, month, end_month) = query.normalized_params()?;

    if !period.is_empty() {
        let query_year = parse_period_year(period)?;
        ensure_period_year_in_range(game, period, query_year, allowed_start, allowed_end)?;
        return Ok(());
    }

    let query_start = YearMonth::parse_yyyy_mm(month)?;
    let query_end = YearMonth::parse_yyyy_mm(end_month)?;
    ensure_month_range_in_range(
        game,
        month,
        end_month,
        query_start,
        query_end,
        allowed_start,
        allowed_end,
    )?;

    Ok(())
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
    fn parse_open_date_to_year_month_accepts_valid_date() {
        let month = parse_open_date_to_year_month("2026-07-08").expect("must parse open_date");
        assert_eq!(month, YearMonth::new(2026, 7));
    }

    #[test]
    fn parse_open_date_to_year_month_rejects_invalid_date() {
        let err =
            parse_open_date_to_year_month("2026-02-30").expect_err("invalid day should be rejected");
        assert!(matches!(err, DownloadError::Io(_)));
    }

    #[test]
    fn parse_date_month_supports_dash_and_slash() {
        assert_eq!(parse_date_month("2026-07-07"), Some("2026-07".to_string()));
        assert_eq!(parse_date_month("2026/07/07"), Some("2026-07".to_string()));
        assert_eq!(parse_date_month("2026"), None);
    }
}
