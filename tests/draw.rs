use taiwan_lottery::{draw_by_game, LotteryGame};

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
    assert_eq!(result.numbers().len(), 6);
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

#[test]
fn test_draw_bingo_bingo() {
    let result = draw_by_game(LotteryGame::BingoBingo);
    assert_eq!(result.numbers().len(), 20);
    assert!(result.bonus.is_some());
    assert!(result.numbers().iter().all(|n| *n >= 1 && *n <= 80));
    assert!(result.bonus.unwrap() >= 1 && result.bonus.unwrap() <= 80);
    assert!(!result.numbers().contains(&result.bonus.unwrap()));
}
