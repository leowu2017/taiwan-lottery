use crate::BonusDrawNumbers;
use crate::LotteryGame;
use rand::seq::SliceRandom;
use rand::Rng;

/// Draw result for a random draw outcome.
pub type DrawResult = BonusDrawNumbers;

fn draw_unique_numbers(rng: &mut impl Rng, min: i32, max: i32, picks: usize) -> Vec<i32> {
    let mut numbers: Vec<i32> = (min..=max).collect();
    numbers.shuffle(rng);
    numbers.into_iter().take(picks).collect()
}

fn draw_bonus_number(rng: &mut impl Rng, min: i32, max: i32) -> i32 {
    rng.gen_range(min..=max)
}

fn draw_numbers_for_rule(
    rng: &mut impl Rng,
    min: i32,
    max: i32,
    picks: usize,
    allow_repeat: bool,
) -> Vec<i32> {
    if allow_repeat {
        (0..picks).map(|_| rng.gen_range(min..=max)).collect()
    } else {
        draw_unique_numbers(rng, min, max, picks)
    }
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
    let metadata = game.metadata();
    let mut base_numbers = Vec::new();
    let mut bonus = None;
    let mut main_range: Option<(i32, i32)> = None;

    for rule in metadata.number_ranges {
        if main_range.is_none() {
            main_range = Some((rule.min, rule.max));
        }

        if (rule.name == "bonus" || rule.name == "super") && rule.picks == 1 {
            let value = if !rule.allow_repeat && Some((rule.min, rule.max)) == main_range {
                let mut candidates: Vec<i32> = (rule.min..=rule.max)
                    .filter(|value| !base_numbers.contains(value))
                    .collect();
                candidates.shuffle(&mut rng);
                candidates
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| draw_bonus_number(&mut rng, rule.min, rule.max))
            } else {
                draw_bonus_number(&mut rng, rule.min, rule.max)
            };
            bonus = Some(value);
            continue;
        }

        base_numbers.extend(draw_numbers_for_rule(
            &mut rng,
            rule.min,
            rule.max,
            rule.picks,
            rule.allow_repeat,
        ));
    }

    DrawResult::new(base_numbers, bonus)
}
