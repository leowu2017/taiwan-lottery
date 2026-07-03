use crate::BonusDrawNumbers;
use crate::HistoryGame;
use rand::seq::SliceRandom;

/// Draw result for a random draw outcome.
pub type DrawResult = BonusDrawNumbers;

/// Perform a random draw based on the game type
pub fn draw_by_game(game: HistoryGame) -> DrawResult {
    let mut rng = rand::thread_rng();

    match game {
        // Lotto649: Select 6 from 1-49, plus 1 bonus from 1-49
        HistoryGame::Lotto649 => {
            let mut numbers: Vec<i32> = (1..=49).collect();
            numbers.shuffle(&mut rng);
            let drawn: Vec<i32> = numbers.iter().take(6).copied().collect();
            let bonus = numbers[6];
            DrawResult::new(drawn, Some(bonus))
        }

        // SuperLotto638: Select 6 from 1-49, plus 1 bonus from 1-8
        HistoryGame::SuperLotto638 => {
            let mut numbers: Vec<i32> = (1..=49).collect();
            numbers.shuffle(&mut rng);
            let drawn: Vec<i32> = numbers.iter().take(6).copied().collect();
            let bonus: i32 = (1..=8)
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap();
            DrawResult::new(drawn, Some(bonus))
        }

        // Daily539: Select 5 from 1-39, plus 1 bonus from 1-39
        HistoryGame::Daily539 => {
            let mut numbers: Vec<i32> = (1..=39).collect();
            numbers.shuffle(&mut rng);
            let drawn: Vec<i32> = numbers.iter().take(5).copied().collect();
            let bonus = numbers[5];
            DrawResult::new(drawn, Some(bonus))
        }

        // Lotto3D: Randomly select 3 digits from 0-9
        HistoryGame::Lotto3D => {
            let num1 = (0..=9)
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap();
            let num2 = (0..=9)
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap();
            let num3 = (0..=9)
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap();
            DrawResult::new(vec![num1, num2, num3], None)
        }

        // Lotto4D: Randomly select 4 digits from 0-9
        HistoryGame::Lotto4D => {
            let num1 = (0..=9)
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap();
            let num2 = (0..=9)
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap();
            let num3 = (0..=9)
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap();
            let num4 = (0..=9)
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap();
            DrawResult::new(vec![num1, num2, num3, num4], None)
        }

        // Lotto49M6: Select 6 from 1-49
        HistoryGame::Lotto49M6 => {
            let mut numbers: Vec<i32> = (1..=49).collect();
            numbers.shuffle(&mut rng);
            let drawn: Vec<i32> = numbers.iter().take(6).copied().collect();
            DrawResult::new(drawn, None)
        }

        // Lotto39M5: Select 5 from 1-39
        HistoryGame::Lotto39M5 => {
            let mut numbers: Vec<i32> = (1..=39).collect();
            numbers.shuffle(&mut rng);
            let drawn: Vec<i32> = numbers.iter().take(5).copied().collect();
            DrawResult::new(drawn, None)
        }

        // Lotto38M6: Select 6 from 1-38
        HistoryGame::Lotto38M6 => {
            let mut numbers: Vec<i32> = (1..=38).collect();
            numbers.shuffle(&mut rng);
            let drawn: Vec<i32> = numbers.iter().take(6).copied().collect();
            DrawResult::new(drawn, None)
        }

        // Lotto1224: Select 2 from 1-18, 2 from 19-27, 2 from 28-36
        HistoryGame::Lotto1224 => {
            let mut zone1: Vec<i32> = (1..=18).collect();
            zone1.shuffle(&mut rng);
            let draw1: Vec<i32> = zone1.iter().take(2).copied().collect();

            let mut zone2: Vec<i32> = (19..=27).collect();
            zone2.shuffle(&mut rng);
            let draw2: Vec<i32> = zone2.iter().take(2).copied().collect();

            let mut zone3: Vec<i32> = (28..=36).collect();
            zone3.shuffle(&mut rng);
            let draw3: Vec<i32> = zone3.iter().take(2).copied().collect();

            let mut all = draw1;
            all.extend(draw2);
            all.extend(draw3);
            DrawResult::new(all, None)
        }

        // Lotto740: Select 7 from 1-38, plus 1 bonus from 1-8
        HistoryGame::Lotto740 => {
            let mut numbers: Vec<i32> = (1..=38).collect();
            numbers.shuffle(&mut rng);
            let drawn: Vec<i32> = numbers.iter().take(7).copied().collect();
            let bonus: i32 = (1..=8)
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap();
            DrawResult::new(drawn, Some(bonus))
        }

        // TicTacToe: Select 20 from 1-80
        HistoryGame::TicTacToe => {
            let mut numbers: Vec<i32> = (1..=80).collect();
            numbers.shuffle(&mut rng);
            let drawn: Vec<i32> = numbers.iter().take(20).copied().collect();
            DrawResult::new(drawn, None)
        }

        // Lotto638: Select 6 from 1-49, plus 1 bonus from 1-10
        HistoryGame::Lotto638 => {
            let mut numbers: Vec<i32> = (1..=49).collect();
            numbers.shuffle(&mut rng);
            let drawn: Vec<i32> = numbers.iter().take(6).copied().collect();
            let bonus: i32 = (1..=10)
                .collect::<Vec<_>>()
                .choose(&mut rng)
                .copied()
                .unwrap();
            DrawResult::new(drawn, Some(bonus))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_lotto649() {
        let result = draw_by_game(HistoryGame::Lotto649);
        assert_eq!(result.numbers().len(), 6);
        assert!(result.bonus.is_some());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 49));
        assert!(result.bonus.unwrap() >= 1 && result.bonus.unwrap() <= 49);
    }

    #[test]
    fn test_draw_daily539() {
        let result = draw_by_game(HistoryGame::Daily539);
        assert_eq!(result.numbers().len(), 5);
        assert!(result.bonus.is_some());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 39));
    }

    #[test]
    fn test_draw_3d() {
        let result = draw_by_game(HistoryGame::Lotto3D);
        assert_eq!(result.numbers().len(), 3);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 0 && *n <= 9));
    }

    #[test]
    fn test_draw_4d() {
        let result = draw_by_game(HistoryGame::Lotto4D);
        assert_eq!(result.numbers().len(), 4);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 0 && *n <= 9));
    }

    #[test]
    fn test_draw_1224() {
        let result = draw_by_game(HistoryGame::Lotto1224);
        assert_eq!(result.numbers().len(), 6); // 2+2+2
        assert!(result.bonus.is_none());
    }

    #[test]
    fn test_draw_tic_tac_toe() {
        let result = draw_by_game(HistoryGame::TicTacToe);
        assert_eq!(result.numbers().len(), 20);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 80));
    }

    #[test]
    fn test_draw_super_lotto638() {
        let result = draw_by_game(HistoryGame::SuperLotto638);
        assert_eq!(result.numbers().len(), 6);
        assert!(result.bonus.is_some());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 49));
        assert!(result.bonus.unwrap() >= 1 && result.bonus.unwrap() <= 8);
    }

    #[test]
    fn test_draw_lotto49m6() {
        let result = draw_by_game(HistoryGame::Lotto49M6);
        assert_eq!(result.numbers().len(), 6);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 49));
    }

    #[test]
    fn test_draw_lotto39m5() {
        let result = draw_by_game(HistoryGame::Lotto39M5);
        assert_eq!(result.numbers().len(), 5);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 39));
    }

    #[test]
    fn test_draw_lotto38m6() {
        let result = draw_by_game(HistoryGame::Lotto38M6);
        assert_eq!(result.numbers().len(), 6);
        assert!(result.bonus.is_none());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 38));
    }

    #[test]
    fn test_draw_lotto740() {
        let result = draw_by_game(HistoryGame::Lotto740);
        assert_eq!(result.numbers().len(), 7);
        assert!(result.bonus.is_some());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 38));
        assert!(result.bonus.unwrap() >= 1 && result.bonus.unwrap() <= 8);
    }

    #[test]
    fn test_draw_lotto638() {
        let result = draw_by_game(HistoryGame::Lotto638);
        assert_eq!(result.numbers().len(), 6);
        assert!(result.bonus.is_some());
        assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 49));
        assert!(result.bonus.unwrap() >= 1 && result.bonus.unwrap() <= 10);
    }
}
