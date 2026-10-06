use crate::rule::{
    query_date_range_for_game, query_date_range_for_game_with_sources, GameQueryDateRange,
};
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
            .ok_or_else(|| DownloadError::invalid_query("month must be in YYYY-MM format"))?
            .parse::<i32>()
            .map_err(|_| DownloadError::invalid_query("month year must be a valid number"))?;
        let month = parts
            .next()
            .ok_or_else(|| DownloadError::invalid_query("month must be in YYYY-MM format"))?
            .parse::<u8>()
            .map_err(|_| DownloadError::invalid_query("month must be a valid number"))?;

        if parts.next().is_some() {
            return Err(DownloadError::invalid_query(
                "month must be in YYYY-MM format",
            ));
        }
        if !(1..=12).contains(&month) {
            return Err(DownloadError::invalid_query(
                "month must be between 01 and 12",
            ));
        }

        Ok(Self::new(year, month))
    }

    pub(crate) fn to_yyyy_mm(self) -> String {
        format!("{:04}-{:02}", self.year, self.month)
    }
}

/// Represents a year-month-day triple for date range queries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct YearMonthDay {
    pub(crate) year: i32,
    pub(crate) month: u8,
    pub(crate) day: u8,
}

impl YearMonthDay {
    pub(crate) const fn new(year: i32, month: u8, day: u8) -> Self {
        Self { year, month, day }
    }

    pub(crate) fn parse_yyyy_mm_dd(value: &str) -> Result<Self, DownloadError> {
        let trimmed = value.trim();
        let mut parts = trimmed.split('-');
        let year = parts
            .next()
            .ok_or_else(|| DownloadError::invalid_query("date must be in YYYY-MM-DD format"))?
            .parse::<i32>()
            .map_err(|_| DownloadError::invalid_query("date year must be a valid number"))?;
        let month = parts
            .next()
            .ok_or_else(|| DownloadError::invalid_query("date must be in YYYY-MM-DD format"))?
            .parse::<u8>()
            .map_err(|_| DownloadError::invalid_query("date month must be a valid number"))?;
        let day = parts
            .next()
            .ok_or_else(|| DownloadError::invalid_query("date must be in YYYY-MM-DD format"))?
            .parse::<u8>()
            .map_err(|_| DownloadError::invalid_query("date day must be a valid number"))?;

        if parts.next().is_some() {
            return Err(DownloadError::invalid_query(
                "date must be in YYYY-MM-DD format",
            ));
        }
        if !(1..=12).contains(&month) {
            return Err(DownloadError::invalid_query(
                "date month must be between 01 and 12",
            ));
        }
        let max_day = days_in_month(year, month);
        if day == 0 || day > max_day {
            return Err(DownloadError::invalid_query(format!(
                "date day must be between 01 and {max_day:02}"
            )));
        }

        Ok(Self::new(year, month, day))
    }

    pub(crate) fn to_yyyy_mm_dd(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    pub(crate) fn to_year_month(self) -> YearMonth {
        YearMonth::new(self.year, self.month)
    }
}

#[cfg(test)]
pub(crate) fn current_utc_year_month() -> YearMonth {
    let now = time::OffsetDateTime::now_utc();
    YearMonth::new(now.year(), u8::from(now.month()))
}

/// Orders periods newest first; longer periods are newer, equal lengths compare lexicographically.
pub(crate) fn period_newest_first(left: &str, right: &str) -> std::cmp::Ordering {
    (right.len(), right).cmp(&(left.len(), left))
}

pub(crate) fn parse_period_year(period: &str) -> Result<i32, DownloadError> {
    let trimmed = period.trim();
    if trimmed.len() < 3 {
        return Err(DownloadError::invalid_query(
            "period must include at least 3 ROC year digits",
        ));
    }

    let roc_year = trimmed
        .chars()
        .take(3)
        .collect::<String>()
        .parse::<i32>()
        .map_err(|_| DownloadError::invalid_query("period must start with 3 ROC year digits"))?;
    Ok(roc_year + 1911)
}

pub(crate) fn parse_open_date_to_year_month(open_date: &str) -> Result<YearMonth, DownloadError> {
    YearMonthDay::parse_yyyy_mm_dd(open_date).map(YearMonthDay::to_year_month)
}

pub(crate) fn parse_date_year_month(date: &str) -> Option<YearMonth> {
    let normalized = date.trim().replace('/', "-");
    let mut parts = normalized.split('-');
    let year = parts.next()?.parse::<i32>().ok()?;
    let month = parts.next()?.parse::<u8>().ok()?;
    (1..=12)
        .contains(&month)
        .then(|| YearMonth::new(year, month))
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
    let (start, end) = range_bounds(query_date_range_for_game(game));
    (start.to_year_month(), end.to_year_month())
}

fn range_bounds(range: GameQueryDateRange) -> (YearMonthDay, YearMonthDay) {
    let start = YearMonthDay::new(range.start_year, range.start_month, range.start_day);
    let end = match (range.end_year, range.end_month, range.end_day) {
        (Some(year), Some(month), Some(day)) => YearMonthDay::new(year, month, day),
        // Active game: cap to today so future dates are rejected.
        _ => {
            let now = time::OffsetDateTime::now_utc();
            YearMonthDay::new(now.year(), u8::from(now.month()), now.day())
        }
    };

    (start, end)
}

pub(crate) fn game_query_date_bounds(game: LotteryGame) -> (YearMonthDay, YearMonthDay) {
    range_bounds(query_date_range_for_game(game))
}

pub(crate) fn game_query_date_bounds_for_local(game: LotteryGame) -> (YearMonthDay, YearMonthDay) {
    range_bounds(query_date_range_for_game_with_sources(game).local)
}

pub(crate) fn game_query_date_bounds_for_remote(game: LotteryGame) -> (YearMonthDay, YearMonthDay) {
    range_bounds(query_date_range_for_game_with_sources(game).remote)
}

fn ensure_period_year_in_range(
    game: LotteryGame,
    period: &str,
    query_year: i32,
    allowed_start: YearMonth,
    allowed_end: YearMonth,
) -> Result<(), DownloadError> {
    if query_year < allowed_start.year || query_year > allowed_end.year {
        return Err(DownloadError::invalid_query(format!(
            "query period {period} (AD {query_year}) is outside supported range {}-{:02} to {}-{:02} for {}",
            allowed_start.year,
            allowed_start.month,
            allowed_end.year,
            allowed_end.month,
            game.metadata().display_name
        )));
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
        return Err(DownloadError::invalid_query(
            "end_month must be greater than or equal to month",
        ));
    }

    if query_start < allowed_start || query_end > allowed_end {
        return Err(DownloadError::invalid_query(format!(
            "query month range {} to {} is outside supported range {}-{:02} to {}-{:02} for {}",
            month,
            end_month,
            allowed_start.year,
            allowed_start.month,
            allowed_end.year,
            allowed_end.month,
            game.metadata().display_name
        )));
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
            return Err(DownloadError::invalid_query(
                "open_date or period is required for BINGO BINGO queries",
            ));
        }

        let query_month = parse_open_date_to_year_month(open_date)?;
        if query_month < allowed_start || query_month > allowed_end {
            return Err(DownloadError::invalid_query(format!(
                "query open_date {open_date} is outside supported range {}-{:02} to {}-{:02} for {}",
                allowed_start.year,
                allowed_start.month,
                allowed_end.year,
                allowed_end.month,
                game.metadata().display_name
            )));
        }

        return Ok(());
    }

    if query
        .open_date
        .as_deref()
        .map(str::trim)
        .is_some_and(|value| !value.is_empty())
    {
        return Err(DownloadError::invalid_query(
            "open_date is not supported for this game",
        ));
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
        let err = parse_open_date_to_year_month("2026-02-30")
            .expect_err("invalid day should be rejected");
        assert!(matches!(err, DownloadError::InvalidQuery(_)));
    }

    #[test]
    fn period_newest_first_orders_descending_by_width_then_text() {
        let mut periods = vec!["114000104", "115000001", "99000001", "114000100"];
        periods.sort_by(|left, right| period_newest_first(left, right));
        assert_eq!(
            periods,
            vec!["115000001", "114000104", "114000100", "99000001"]
        );
    }

    #[test]
    fn parse_date_year_month_supports_dash_and_slash() {
        let expected = Some(YearMonth::new(2026, 7));
        assert_eq!(parse_date_year_month("2026-07-07"), expected);
        assert_eq!(parse_date_year_month("2026/07/07"), expected);
        assert_eq!(parse_date_year_month("2026"), None);
    }

    #[test]
    fn parse_date_year_month_rejects_non_ascii_and_invalid_months() {
        assert_eq!(parse_date_year_month("二〇二六年七月七日"), None);
        assert_eq!(parse_date_year_month("2026-13-01"), None);
        assert_eq!(parse_date_year_month(""), None);
    }
}
