use crate::NumberSegmentKind::{Base, Bonus};
use crate::{LotteryGame, LotteryGameMetadata, LotteryGameNumberRule};

/// Query date range for a lottery game (includes day precision).
/// start: (year, month, day) of the first available draw
/// end: (year, month, day) of the last available draw; if None, game is active (querying up to current month)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GameQueryDateRange {
    pub(crate) start_year: i32,
    pub(crate) start_month: u8,
    pub(crate) start_day: u8,
    pub(crate) end_year: Option<i32>,
    pub(crate) end_month: Option<u8>,
    pub(crate) end_day: Option<u8>,
}

/// Query date range with separate local (D423F) and remote (Taiwan Lottery API) ranges.
/// Used when the two sources have different coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct GameQueryDateRangeWithSources {
    pub(crate) local: GameQueryDateRange,
    pub(crate) remote: GameQueryDateRange,
}

impl GameQueryDateRange {
    pub(crate) const fn active(start_year: i32, start_month: u8, start_day: u8) -> Self {
        Self {
            start_year,
            start_month,
            start_day,
            end_year: None,
            end_month: None,
            end_day: None,
        }
    }

    pub(crate) const fn discontinued(
        start_year: i32,
        start_month: u8,
        start_day: u8,
        end_year: i32,
        end_month: u8,
        end_day: u8,
    ) -> Self {
        Self {
            start_year,
            start_month,
            start_day,
            end_year: Some(end_year),
            end_month: Some(end_month),
            end_day: Some(end_day),
        }
    }
}

impl GameQueryDateRangeWithSources {
    pub(crate) const fn new(local: GameQueryDateRange, remote: GameQueryDateRange) -> Self {
        Self { local, remote }
    }

    pub(crate) const fn same(range: GameQueryDateRange) -> Self {
        Self {
            local: range,
            remote: range,
        }
    }
}

const SUPER_LOTTO_638_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule::new("main", Base, 6, 1, 49, false),
    LotteryGameNumberRule::new("bonus", Bonus, 1, 1, 8, false),
];

const LOTTO_649_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule::new("main", Base, 6, 1, 49, false),
    LotteryGameNumberRule::new("bonus", Bonus, 1, 1, 49, false),
];

const DAILY_539_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule::new("main", Base, 5, 1, 39, false),
    LotteryGameNumberRule::new("bonus", Bonus, 1, 1, 39, false),
];

const LOTTO_3D_NUMBER_RULES: [LotteryGameNumberRule; 1] =
    [LotteryGameNumberRule::new("digits", Base, 3, 0, 9, true)];

const LOTTO_4D_NUMBER_RULES: [LotteryGameNumberRule; 1] =
    [LotteryGameNumberRule::new("digits", Base, 4, 0, 9, true)];

const LOTTO_49M6_NUMBER_RULES: [LotteryGameNumberRule; 1] =
    [LotteryGameNumberRule::new("main", Base, 6, 1, 49, false)];

const LOTTO_39M5_NUMBER_RULES: [LotteryGameNumberRule; 1] =
    [LotteryGameNumberRule::new("main", Base, 5, 1, 39, false)];

const LOTTO_38M6_NUMBER_RULES: [LotteryGameNumberRule; 1] =
    [LotteryGameNumberRule::new("main", Base, 6, 1, 38, false)];

const LOTTO_1224_NUMBER_RULES: [LotteryGameNumberRule; 3] = [
    LotteryGameNumberRule::new("zone_1", Base, 2, 1, 18, false),
    LotteryGameNumberRule::new("zone_2", Base, 2, 19, 27, false),
    LotteryGameNumberRule::new("zone_3", Base, 2, 28, 36, false),
];

const LOTTO_740_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule::new("main", Base, 7, 1, 38, false),
    LotteryGameNumberRule::new("bonus", Bonus, 1, 1, 8, false),
];

const TIC_TAC_TOE_NUMBER_RULES: [LotteryGameNumberRule; 1] =
    [LotteryGameNumberRule::new("main", Base, 20, 1, 80, false)];

const LOTTO_638_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule::new("main", Base, 6, 1, 49, false),
    LotteryGameNumberRule::new("bonus", Bonus, 1, 1, 10, false),
];

const BINGO_BINGO_NUMBER_RULES: [LotteryGameNumberRule; 2] = [
    LotteryGameNumberRule::new("main", Base, 20, 1, 80, false),
    LotteryGameNumberRule::new("super", Bonus, 1, 1, 80, false),
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

pub(crate) const fn query_date_range_for_game(game: LotteryGame) -> GameQueryDateRange {
    match game {
        LotteryGame::SuperLotto638 => GameQueryDateRange::active(2008, 1, 24),
        LotteryGame::Lotto649 => GameQueryDateRange::active(2007, 1, 2),
        LotteryGame::Daily539 => GameQueryDateRange::active(2007, 1, 1),
        LotteryGame::Lotto3D => GameQueryDateRange::active(2007, 1, 1),
        LotteryGame::Lotto4D => GameQueryDateRange::active(2007, 1, 1),
        LotteryGame::Lotto49M6 => GameQueryDateRange::active(2007, 1, 2),
        LotteryGame::Lotto39M5 => GameQueryDateRange::active(2010, 9, 6),
        LotteryGame::BingoBingo => GameQueryDateRange::active(2024, 1, 1),
        LotteryGame::Lotto38M6 => GameQueryDateRange::discontinued(2007, 1, 1, 2023, 12, 28),
        LotteryGame::Lotto638 => GameQueryDateRange::discontinued(2007, 1, 1, 2008, 1, 21),
        LotteryGame::TicTacToe => GameQueryDateRange::discontinued(2009, 7, 27, 2013, 12, 31),
        LotteryGame::Lotto1224 => GameQueryDateRange::discontinued(2018, 4, 23, 2023, 12, 30),
        LotteryGame::Lotto740 => GameQueryDateRange::discontinued(2015, 4, 22, 2019, 4, 27),
    }
}

pub(crate) const fn query_date_range_for_game_with_sources(
    game: LotteryGame,
) -> GameQueryDateRangeWithSources {
    // Based on GAME_MAPPING.md
    // Most games have identical local (D423F) and remote (Taiwan Lottery API) ranges.
    // BingoBingo is the exception: D423F has data from 2008-04-30, remote API only from 2024-01-01.
    match game {
        LotteryGame::SuperLotto638 => {
            GameQueryDateRangeWithSources::same(GameQueryDateRange::active(2008, 1, 24))
        }
        LotteryGame::Lotto649 => {
            GameQueryDateRangeWithSources::same(GameQueryDateRange::active(2007, 1, 2))
        }
        LotteryGame::Daily539 => {
            GameQueryDateRangeWithSources::same(GameQueryDateRange::active(2007, 1, 1))
        }
        LotteryGame::Lotto3D => {
            GameQueryDateRangeWithSources::same(GameQueryDateRange::active(2007, 1, 1))
        }
        LotteryGame::Lotto4D => {
            GameQueryDateRangeWithSources::same(GameQueryDateRange::active(2007, 1, 1))
        }
        LotteryGame::Lotto49M6 => {
            GameQueryDateRangeWithSources::same(GameQueryDateRange::active(2007, 1, 2))
        }
        LotteryGame::Lotto39M5 => {
            GameQueryDateRangeWithSources::same(GameQueryDateRange::active(2010, 9, 6))
        }
        // BingoBingo: Local D423F starts 2008-04-30, Remote API starts 2024-01-01
        LotteryGame::BingoBingo => GameQueryDateRangeWithSources::new(
            GameQueryDateRange::active(2008, 4, 30),
            GameQueryDateRange::active(2024, 1, 1),
        ),
        LotteryGame::Lotto38M6 => GameQueryDateRangeWithSources::same(
            GameQueryDateRange::discontinued(2007, 1, 1, 2023, 12, 28),
        ),
        LotteryGame::Lotto638 => GameQueryDateRangeWithSources::same(
            GameQueryDateRange::discontinued(2007, 1, 1, 2008, 1, 21),
        ),
        LotteryGame::TicTacToe => GameQueryDateRangeWithSources::same(
            GameQueryDateRange::discontinued(2009, 7, 27, 2013, 12, 31),
        ),
        LotteryGame::Lotto1224 => GameQueryDateRangeWithSources::same(
            GameQueryDateRange::discontinued(2018, 4, 23, 2023, 12, 30),
        ),
        LotteryGame::Lotto740 => GameQueryDateRangeWithSources::same(
            GameQueryDateRange::discontinued(2015, 4, 22, 2019, 4, 27),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LotteryDisplayLanguage;

    #[test]
    fn metadata_for_game_exposes_ui_fields() {
        let metadata = metadata_for_game(LotteryGame::Lotto649);
        assert_eq!(metadata.display_name, "Lotto 649");
        assert_eq!(metadata.display_name_english, "Lotto 649");
        assert_eq!(metadata.display_name_chinese, "大樂透");
        assert_eq!(
            metadata.number_rule,
            "6 numbers from 1-49, plus 1 bonus number from 1-49"
        );
        assert_eq!(metadata.number_ranges.len(), 2);
        assert_eq!(metadata.number_ranges[0].name, "main");
        assert_eq!(metadata.number_ranges[0].picks, 6);
        assert_eq!(metadata.number_ranges[0].min, 1);
        assert_eq!(metadata.number_ranges[0].max, 49);
        assert!(!metadata.number_ranges[0].allow_repeat);
    }

    #[test]
    fn metadata_supports_display_language_selection() {
        let default_metadata = metadata_for_game(LotteryGame::Lotto649)
            .with_display_language(LotteryDisplayLanguage::English);
        assert_eq!(default_metadata.display_name, "Lotto 649");

        let chinese_metadata = metadata_for_game(LotteryGame::Lotto649)
            .with_display_language(LotteryDisplayLanguage::Chinese);
        assert_eq!(chinese_metadata.display_name, "大樂透");
    }

    #[test]
    fn query_date_range_active_games() {
        let lotto649_range = query_date_range_for_game(LotteryGame::Lotto649);
        assert_eq!(lotto649_range.start_year, 2007);
        assert_eq!(lotto649_range.start_month, 1);
        assert!(lotto649_range.end_year.is_none());
        assert!(lotto649_range.end_month.is_none());
    }

    #[test]
    fn query_date_range_discontinued_games() {
        let lotto638_range = query_date_range_for_game(LotteryGame::Lotto638);
        assert_eq!(lotto638_range.start_year, 2007);
        assert_eq!(lotto638_range.start_month, 1);
        assert_eq!(lotto638_range.end_year, Some(2008));
        assert_eq!(lotto638_range.end_month, Some(1));
    }

    #[test]
    fn bonus_segments_are_single_picks_named_bonus_or_super() {
        for game in LotteryGame::ALL {
            for rule in metadata_for_game(game).number_ranges {
                let named_bonus = matches!(rule.name, "bonus" | "super");
                assert_eq!(rule.kind == Bonus, named_bonus, "{game:?} {}", rule.name);
                if rule.kind == Bonus {
                    assert_eq!(rule.picks, 1, "{game:?} {}", rule.name);
                }
            }
        }
    }
}
