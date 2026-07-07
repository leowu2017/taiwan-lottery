use crate::{
    DownloadError, HistoryDrawQuery, LotteryGame, LotteryGameMetadata, LotteryGameNumberRule,
};

const THIRD_TERM_START_YEAR: i32 = 2007;
const THIRD_TERM_END_YEAR: i32 = 2013;
const FOURTH_TERM_START_YEAR: i32 = 2014;
const FOURTH_TERM_END_YEAR: i32 = 2023;
const FIFTH_TERM_END_YEAR: i32 = 2033;
const BINGO_BINGO_START_YEAR: i32 = 2024;

const SUPER_LOTTO_638_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule {
        name: "main",
        picks: 6,
        min: 1,
        max: 49,
        allow_repeat: false,
    },
    LotteryGameNumberRule {
        name: "bonus",
        picks: 1,
        min: 1,
        max: 8,
        allow_repeat: false,
    },
];

const LOTTO_649_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule {
        name: "main",
        picks: 6,
        min: 1,
        max: 49,
        allow_repeat: false,
    },
    LotteryGameNumberRule {
        name: "bonus",
        picks: 1,
        min: 1,
        max: 49,
        allow_repeat: false,
    },
];

const DAILY_539_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule {
        name: "main",
        picks: 5,
        min: 1,
        max: 39,
        allow_repeat: false,
    },
    LotteryGameNumberRule {
        name: "bonus",
        picks: 1,
        min: 1,
        max: 39,
        allow_repeat: false,
    },
];

const LOTTO_3D_NUMBER_RULES: [LotteryGameNumberRule; 1] = [LotteryGameNumberRule {
    name: "digits",
    picks: 3,
    min: 0,
    max: 9,
    allow_repeat: true,
}];

const LOTTO_4D_NUMBER_RULES: [LotteryGameNumberRule; 1] = [LotteryGameNumberRule {
    name: "digits",
    picks: 4,
    min: 0,
    max: 9,
    allow_repeat: true,
}];

const LOTTO_49M6_NUMBER_RULES: [LotteryGameNumberRule; 1] = [LotteryGameNumberRule {
    name: "main",
    picks: 6,
    min: 1,
    max: 49,
    allow_repeat: false,
}];

const LOTTO_39M5_NUMBER_RULES: [LotteryGameNumberRule; 1] = [LotteryGameNumberRule {
    name: "main",
    picks: 5,
    min: 1,
    max: 39,
    allow_repeat: false,
}];

const LOTTO_38M6_NUMBER_RULES: [LotteryGameNumberRule; 1] = [LotteryGameNumberRule {
    name: "main",
    picks: 6,
    min: 1,
    max: 38,
    allow_repeat: false,
}];

const LOTTO_1224_NUMBER_RULES: [LotteryGameNumberRule; 3] = [
    LotteryGameNumberRule {
        name: "zone_1",
        picks: 2,
        min: 1,
        max: 18,
        allow_repeat: false,
    },
    LotteryGameNumberRule {
        name: "zone_2",
        picks: 2,
        min: 19,
        max: 27,
        allow_repeat: false,
    },
    LotteryGameNumberRule {
        name: "zone_3",
        picks: 2,
        min: 28,
        max: 36,
        allow_repeat: false,
    },
];

const LOTTO_740_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule {
        name: "main",
        picks: 7,
        min: 1,
        max: 38,
        allow_repeat: false,
    },
    LotteryGameNumberRule {
        name: "bonus",
        picks: 1,
        min: 1,
        max: 8,
        allow_repeat: false,
    },
];

const TIC_TAC_TOE_NUMBER_RULES: [LotteryGameNumberRule; 1] = [LotteryGameNumberRule {
    name: "main",
    picks: 20,
    min: 1,
    max: 80,
    allow_repeat: false,
}];

const LOTTO_638_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule {
        name: "main",
        picks: 6,
        min: 1,
        max: 49,
        allow_repeat: false,
    },
    LotteryGameNumberRule {
        name: "bonus",
        picks: 1,
        min: 1,
        max: 10,
        allow_repeat: false,
    },
];

const BINGO_BINGO_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule {
        name: "main",
        picks: 20,
        min: 1,
        max: 80,
        allow_repeat: false,
    },
    LotteryGameNumberRule {
        name: "super",
        picks: 1,
        min: 1,
        max: 80,
        allow_repeat: false,
    },
];

pub(crate) const fn metadata_for_game(game: LotteryGame) -> LotteryGameMetadata {
    match game {
        LotteryGame::SuperLotto638 => LotteryGameMetadata {
            display_name: "Super Lotto 638",
            display_name_english: "Super Lotto 638",
            display_name_chinese: "威力彩",
            number_rule: "6 numbers from 1-49, plus 1 bonus number from 1-8",
            number_ranges: &SUPER_LOTTO_638_NUMBER_RULES,
        },
        LotteryGame::Lotto649 => LotteryGameMetadata {
            display_name: "Lotto 649",
            display_name_english: "Lotto 649",
            display_name_chinese: "大樂透",
            number_rule: "6 numbers from 1-49, plus 1 bonus number from 1-49",
            number_ranges: &LOTTO_649_NUMBER_RULES,
        },
        LotteryGame::Daily539 => LotteryGameMetadata {
            display_name: "Daily 539",
            display_name_english: "Daily 539",
            display_name_chinese: "今彩539",
            number_rule: "5 numbers from 1-39, plus 1 bonus number from 1-39",
            number_ranges: &DAILY_539_NUMBER_RULES,
        },
        LotteryGame::Lotto3D => LotteryGameMetadata {
            display_name: "3D Lotto",
            display_name_english: "3D Lotto",
            display_name_chinese: "3星彩",
            number_rule: "3 digits from 0-9, digits may repeat",
            number_ranges: &LOTTO_3D_NUMBER_RULES,
        },
        LotteryGame::Lotto4D => LotteryGameMetadata {
            display_name: "4D Lotto",
            display_name_english: "4D Lotto",
            display_name_chinese: "4星彩",
            number_rule: "4 digits from 0-9, digits may repeat",
            number_ranges: &LOTTO_4D_NUMBER_RULES,
        },
        LotteryGame::Lotto49M6 => LotteryGameMetadata {
            display_name: "49M6 Lotto",
            display_name_english: "49M6 Lotto",
            display_name_chinese: "49樂合彩",
            number_rule: "6 numbers from 1-49",
            number_ranges: &LOTTO_49M6_NUMBER_RULES,
        },
        LotteryGame::Lotto39M5 => LotteryGameMetadata {
            display_name: "39M5 Lotto",
            display_name_english: "39M5 Lotto",
            display_name_chinese: "39樂合彩",
            number_rule: "5 numbers from 1-39",
            number_ranges: &LOTTO_39M5_NUMBER_RULES,
        },
        LotteryGame::Lotto38M6 => LotteryGameMetadata {
            display_name: "38M6 Lotto",
            display_name_english: "38M6 Lotto",
            display_name_chinese: "38樂合彩",
            number_rule: "6 numbers from 1-38",
            number_ranges: &LOTTO_38M6_NUMBER_RULES,
        },
        LotteryGame::Lotto1224 => LotteryGameMetadata {
            display_name: "Bingo Bingo 12/24 Pick 6",
            display_name_english: "Bingo Bingo 12/24 Pick 6",
            display_name_chinese: "BINGO BINGO 賓果賓果 12/24選6",
            number_rule: "2 numbers from 1-18, 2 numbers from 19-27, and 2 numbers from 28-36",
            number_ranges: &LOTTO_1224_NUMBER_RULES,
        },
        LotteryGame::Lotto740 => LotteryGameMetadata {
            display_name: "Bingo Bingo 7/40",
            display_name_english: "Bingo Bingo 7/40",
            display_name_chinese: "BINGO BINGO 賓果賓果 7/40",
            number_rule: "7 numbers from 1-38, plus 1 bonus number from 1-8",
            number_ranges: &LOTTO_740_NUMBER_RULES,
        },
        LotteryGame::TicTacToe => LotteryGameMetadata {
            display_name: "Bingo Bingo Big/Small Odd/Even",
            display_name_english: "Bingo Bingo Big/Small Odd/Even",
            display_name_chinese: "BINGO BINGO 賓果賓果 猜大小單雙",
            number_rule: "20 numbers from 1-80",
            number_ranges: &TIC_TAC_TOE_NUMBER_RULES,
        },
        LotteryGame::Lotto638 => LotteryGameMetadata {
            display_name: "Lotto 638",
            display_name_english: "Lotto 638",
            display_name_chinese: "6/38樂透彩",
            number_rule: "6 numbers from 1-49, plus 1 bonus number from 1-10",
            number_ranges: &LOTTO_638_NUMBER_RULES,
        },
        LotteryGame::BingoBingo => LotteryGameMetadata {
            display_name: "Bingo Bingo",
            display_name_english: "Bingo Bingo",
            display_name_chinese: "BINGO BINGO 賓果賓果",
            number_rule: "20 numbers from 1-80, plus 1 super number from 1-80",
            number_ranges: &BINGO_BINGO_NUMBER_RULES,
        },
    }
}

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

fn parse_period_year(period: &str) -> Result<i32, DownloadError> {
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

pub(crate) fn game_query_month_bounds(game: LotteryGame) -> (YearMonth, YearMonth) {
    let dynamic_fifth_end = {
        let now = current_utc_year_month();
        let season_end = YearMonth::new(FIFTH_TERM_END_YEAR, 12);
        if now < season_end {
            now
        } else {
            season_end
        }
    };

    match game {
        LotteryGame::SuperLotto638
        | LotteryGame::Lotto649
        | LotteryGame::Daily539
        | LotteryGame::Lotto3D
        | LotteryGame::Lotto4D
        | LotteryGame::Lotto49M6
        | LotteryGame::Lotto39M5 => (YearMonth::new(THIRD_TERM_START_YEAR, 1), dynamic_fifth_end),
        LotteryGame::BingoBingo => (YearMonth::new(BINGO_BINGO_START_YEAR, 1), dynamic_fifth_end),
        LotteryGame::Lotto38M6 => (
            YearMonth::new(THIRD_TERM_START_YEAR, 1),
            YearMonth::new(FOURTH_TERM_END_YEAR, 12),
        ),
        LotteryGame::Lotto1224 | LotteryGame::Lotto740 => (
            YearMonth::new(FOURTH_TERM_START_YEAR, 1),
            YearMonth::new(FOURTH_TERM_END_YEAR, 12),
        ),
        LotteryGame::TicTacToe | LotteryGame::Lotto638 => (
            YearMonth::new(THIRD_TERM_START_YEAR, 1),
            YearMonth::new(THIRD_TERM_END_YEAR, 12),
        ),
    }
}

pub(crate) fn validate_query_range_for_game(
    game: LotteryGame,
    query: &HistoryDrawQuery,
) -> Result<(), DownloadError> {
    let (allowed_start, allowed_end) = game_query_month_bounds(game);
    let (period, month, end_month) = query.normalized_params()?;

    if !period.is_empty() {
        let query_year = parse_period_year(period)?;
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
        return Ok(());
    }

    let query_start = YearMonth::parse_yyyy_mm(month)?;
    let query_end = YearMonth::parse_yyyy_mm(end_month)?;
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
