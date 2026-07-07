/// Base structure shared by draw and history number models.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DrawNumbers {
    /// Primary numbers for the record.
    pub numbers: Vec<i32>,
}

impl DrawNumbers {
    pub fn new(numbers: Vec<i32>) -> Self {
        Self { numbers }
    }

    pub fn as_slice(&self) -> &[i32] {
        &self.numbers
    }
}

/// Draw numbers with an optional bonus number.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BonusDrawNumbers {
    pub base: DrawNumbers,
    pub bonus: Option<i32>,
}

impl BonusDrawNumbers {
    pub fn new(numbers: Vec<i32>, bonus: Option<i32>) -> Self {
        Self {
            base: DrawNumbers::new(numbers),
            bonus,
        }
    }

    pub fn numbers(&self) -> &[i32] {
        self.base.as_slice()
    }

    pub fn all_numbers(&self) -> Vec<i32> {
        let mut all = self.base.numbers.clone();
        if let Some(bonus) = self.bonus {
            all.push(bonus);
        }
        all
    }
}

/// Draw numbers with an optional sorted view.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SortedDrawNumbers {
    pub base: DrawNumbers,
    pub sorted: Option<Vec<i32>>,
}

impl SortedDrawNumbers {
    pub fn new(numbers: Vec<i32>, sorted: Option<Vec<i32>>) -> Self {
        Self {
            base: DrawNumbers::new(numbers),
            sorted,
        }
    }

    pub fn numbers(&self) -> &[i32] {
        self.base.as_slice()
    }

    pub fn sorted_numbers(&self) -> Option<&[i32]> {
        self.sorted.as_deref()
    }
}

/// SuperLotto 638 draw numbers with bonus.
pub type SuperLotto638Numbers = BonusDrawNumbers;
/// Lotto 649 draw numbers with bonus.
pub type Lotto649Numbers = BonusDrawNumbers;
/// Daily 539 draw numbers with bonus.
pub type Daily539Numbers = BonusDrawNumbers;
/// Lotto 3D draw numbers without bonus.
pub type Lotto3DNumbers = DrawNumbers;
/// Lotto 4D draw numbers without bonus.
pub type Lotto4DNumbers = DrawNumbers;
/// Lotto 49/6 draw numbers without bonus.
pub type Lotto49M6Numbers = DrawNumbers;
/// Lotto 39/5 draw numbers without bonus.
pub type Lotto39M5Numbers = DrawNumbers;
/// Lotto 38/6 draw numbers without bonus.
pub type Lotto38M6Numbers = DrawNumbers;
/// Lotto 1224 draw numbers without bonus.
pub type Lotto1224Numbers = DrawNumbers;
/// Lotto 740 draw numbers with bonus.
pub type Lotto740Numbers = BonusDrawNumbers;
/// Tic-Tac-Toe draw numbers without bonus.
pub type TicTacToeNumbers = DrawNumbers;
/// Lotto 638 draw numbers with bonus.
pub type Lotto638Numbers = BonusDrawNumbers;
/// Bingo Bingo draw numbers with super number.
pub type BingoBingoNumbers = BonusDrawNumbers;
