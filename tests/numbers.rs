use taiwan_lottery::{BonusDrawNumbers, DrawNumbers, SortedDrawNumbers};

#[test]
fn test_draw_numbers_creation() {
    let numbers = DrawNumbers::new(vec![1, 2, 3]);
    assert_eq!(numbers.numbers, vec![1, 2, 3]);
}

#[test]
fn test_bonus_draw_numbers_with_bonus() {
    let numbers = BonusDrawNumbers::new(vec![1, 2, 3], Some(4));
    assert_eq!(numbers.numbers(), &[1, 2, 3]);
    assert_eq!(numbers.all_numbers(), vec![1, 2, 3, 4]);
}

#[test]
fn test_bonus_draw_numbers_without_bonus() {
    let numbers = BonusDrawNumbers::new(vec![1, 2, 3], None);
    assert_eq!(numbers.all_numbers(), vec![1, 2, 3]);
}

#[test]
fn test_sorted_draw_numbers() {
    let numbers = SortedDrawNumbers::new(vec![9, 3, 5], Some(vec![3, 5, 9]));
    assert_eq!(numbers.numbers(), &[9, 3, 5]);
    assert_eq!(numbers.sorted_numbers(), Some([3, 5, 9].as_slice()));
}
