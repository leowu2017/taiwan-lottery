use crate::BonusDrawNumbers;
use crate::LotteryGame;
use rand::Rng;
use rand::seq::SliceRandom;

/// Draw result for a random draw outcome.
pub type DrawResult = BonusDrawNumbers;

fn draw_unique_numbers(rng: &mut impl Rng, min: i32, max: i32, picks: usize) -> Vec<i32> {
    let mut numbers: Vec<i32> = (min..=max).collect();
    numbers.shuffle(rng);
    numbers.into_iter().take(picks).collect()
}

fn draw_unique_numbers_with_bonus(
    rng: &mut impl Rng,
    min: i32,
    max: i32,
    picks: usize,
) -> (Vec<i32>, i32) {
    let mut numbers: Vec<i32> = (min..=max).collect();
    numbers.shuffle(rng);
    let bonus = numbers[picks];
    (numbers.into_iter().take(picks).collect(), bonus)
}

fn draw_bonus_number(rng: &mut impl Rng, min: i32, max: i32) -> i32 {
    rng.gen_range(min..=max)
}

fn draw_digit_numbers(rng: &mut impl Rng, picks: usize) -> Vec<i32> {
    (0..picks).map(|_| rng.gen_range(0..=9)).collect()
}

fn draw_partitioned_numbers(rng: &mut impl Rng, partitions: &[(i32, i32, usize)]) -> Vec<i32> {
    let mut numbers = Vec::new();
    for (min, max, picks) in partitions {
        numbers.extend(draw_unique_numbers(rng, *min, *max, *picks));
    }
    numbers
}

/// Perform a random draw based on the game type and return the drawn numbers.
///
/// Generates random numbers according to the game's rules. The returned numbers
/// are not sorted - they appear in draw order. If the game has a bonus number,
/// it is stored in the [`BonusDrawNumbers::bonus`] field.
///
/// # Arguments
/// * `game` - The lottery game to draw for
///
/// # Returns
/// A [`DrawResult`] containing the drawn numbers and optional bonus.
///
/// # Example
/// ```ignore
/// use taiwan_lottery::{draw_by_game, LotteryGame};
///
/// let result = draw_by_game(LotteryGame::Lotto649);
/// println!("Numbers: {:?}", result.base.numbers);
/// if let Some(bonus) = result.bonus {
///     println!("Bonus: {}", bonus);
/// }
/// ```
pub fn draw_by_game(game: LotteryGame) -> DrawResult {
    let mut rng = rand::thread_rng();

    match game {
        // Lotto649: Select 6 from 1-49, plus 1 bonus from 1-49
        LotteryGame::Lotto649 => {
            let (drawn, bonus) = draw_unique_numbers_with_bonus(&mut rng, 1, 49, 6);
            DrawResult::new(drawn, Some(bonus))
        }

        // SuperLotto638: Select 6 from 1-49, plus 1 bonus from 1-8
        LotteryGame::SuperLotto638 => DrawResult::new(
            draw_unique_numbers(&mut rng, 1, 49, 6),
            Some(draw_bonus_number(&mut rng, 1, 8)),
        ),

        // Daily539: Select 5 from 1-39, plus 1 bonus from 1-39
        LotteryGame::Daily539 => {
            let (drawn, bonus) = draw_unique_numbers_with_bonus(&mut rng, 1, 39, 5);
            DrawResult::new(drawn, Some(bonus))
        }

        // Lotto3D: Randomly select 3 digits from 0-9
        LotteryGame::Lotto3D => DrawResult::new(draw_digit_numbers(&mut rng, 3), None),

        // Lotto4D: Randomly select 4 digits from 0-9
        LotteryGame::Lotto4D => DrawResult::new(draw_digit_numbers(&mut rng, 4), None),

        // Lotto49M6: Select 6 from 1-49
        LotteryGame::Lotto49M6 => DrawResult::new(draw_unique_numbers(&mut rng, 1, 49, 6), None),

        // Lotto39M5: Select 5 from 1-39
        LotteryGame::Lotto39M5 => DrawResult::new(draw_unique_numbers(&mut rng, 1, 39, 5), None),

        // Lotto38M6: Select 6 from 1-38
        LotteryGame::Lotto38M6 => DrawResult::new(draw_unique_numbers(&mut rng, 1, 38, 6), None),

        // Lotto1224: Select 2 from 1-18, 2 from 19-27, 2 from 28-36
        LotteryGame::Lotto1224 => DrawResult::new(
            draw_partitioned_numbers(&mut rng, &[(1, 18, 2), (19, 27, 2), (28, 36, 2)]),
            None,
        ),

        // Lotto740: Select 7 from 1-38, plus 1 bonus from 1-8
        LotteryGame::Lotto740 => DrawResult::new(
            draw_unique_numbers(&mut rng, 1, 38, 7),
            Some(draw_bonus_number(&mut rng, 1, 8)),
        ),

        // TicTacToe: Select 20 from 1-80
        LotteryGame::TicTacToe => DrawResult::new(draw_unique_numbers(&mut rng, 1, 80, 20), None),

        // Lotto638: Select 6 from 1-49, plus 1 bonus from 1-10
        LotteryGame::Lotto638 => DrawResult::new(
            draw_unique_numbers(&mut rng, 1, 49, 6),
            Some(draw_bonus_number(&mut rng, 1, 10)),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_lotto649() {
        let result = draw_by_game(LotteryGame::Lotto649);
        assert_eq!(result.numbers().len(), 6);
        assert!(result.bonus.is_some());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 49));
        assert!(result.bonus.unwrap() >= 1 && result.bonus.unwrap() <= 49);
        assert!(!result.numbers().contains(&result.bonus.unwrap()));
    }

    #[test]
    fn test_draw_daily539() {
        let result = draw_by_game(LotteryGame::Daily539);
        assert_eq!(result.numbers().len(), 5);
        assert!(result.bonus.is_some());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 39));
        assert!(!result.numbers().contains(&result.bonus.unwrap()));
    }

    #[test]
    fn test_draw_3d() {
        let result = draw_by_game(LotteryGame::Lotto3D);
        assert_eq!(result.numbers().len(), 3);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 0 && *n <= 9));
    }

    #[test]
    fn test_draw_4d() {
        let result = draw_by_game(LotteryGame::Lotto4D);
        assert_eq!(result.numbers().len(), 4);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 0 && *n <= 9));
    }

    #[test]
    fn test_draw_1224() {
        let result = draw_by_game(LotteryGame::Lotto1224);
        assert_eq!(result.numbers().len(), 6); // 2+2+2
        assert!(result.bonus.is_none());
    }

    #[test]
    fn test_draw_tic_tac_toe() {
        let result = draw_by_game(LotteryGame::TicTacToe);
        assert_eq!(result.numbers().len(), 20);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 80));
    }

    #[test]
    fn test_draw_super_lotto638() {
        let result = draw_by_game(LotteryGame::SuperLotto638);
        assert_eq!(result.numbers().len(), 6);
        assert!(result.bonus.is_some());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 49));
        assert!(result.bonus.unwrap() >= 1 && result.bonus.unwrap() <= 8);
    }

    #[test]
    fn test_draw_lotto49m6() {
        let result = draw_by_game(LotteryGame::Lotto49M6);
        assert_eq!(result.numbers().len(), 6);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 49));
    }

    #[test]
    fn test_draw_lotto39m5() {
        let result = draw_by_game(LotteryGame::Lotto39M5);
        assert_eq!(result.numbers().len(), 5);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 39));
    }

    #[test]
    fn test_draw_lotto38m6() {
        let result = draw_by_game(LotteryGame::Lotto38M6);
        assert_eq!(result.numbers().len(), 6);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 38));
    }

    #[test]
    fn test_draw_lotto740() {
        let result = draw_by_game(LotteryGame::Lotto740);
        assert_eq!(result.numbers().len(), 7);
        assert!(result.bonus.is_some());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 38));
        assert!(result.bonus.unwrap() >= 1 && result.bonus.unwrap() <= 8);
    }

    #[test]
    fn test_draw_lotto638() {
        let result = draw_by_game(LotteryGame::Lotto638);
        assert_eq!(result.numbers().len(), 6);
        assert!(result.bonus.is_some());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 49));
        assert!(result.bonus.unwrap() >= 1 && result.bonus.unwrap() <= 10);
    }
}
