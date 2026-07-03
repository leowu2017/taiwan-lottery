use taiwan_lottery::{HistoryGame, draw_by_game};

fn print_usage(program: &str) {
    eprintln!("Usage:");
    eprintln!("  {program} <game>");
    eprintln!("  {program} <game> <count>");
    eprintln!("");
    eprintln!(
        "game: super-lotto638 | lotto649 | daily539 | 3d | 4d | 49m6 | 39m5 | 38m6 | 1224 | 740 | tic-tac-toe | 638"
    );
    eprintln!("count: number of times to draw (default: 1)");
}

fn parse_game(value: &str) -> Option<HistoryGame> {
    match value.to_ascii_lowercase().as_str() {
        "super-lotto638" | "superlotto638" | "5134" => Some(HistoryGame::SuperLotto638),
        "lotto649" | "5118" => Some(HistoryGame::Lotto649),
        "daily539" | "5120" => Some(HistoryGame::Daily539),
        "3d" | "2108" => Some(HistoryGame::Lotto3D),
        "4d" | "2109" => Some(HistoryGame::Lotto4D),
        "49m6" | "1121" => Some(HistoryGame::Lotto49M6),
        "39m5" | "1197" => Some(HistoryGame::Lotto39M5),
        "38m6" | "5122" => Some(HistoryGame::Lotto38M6),
        "1224" | "5290" => Some(HistoryGame::Lotto1224),
        "740" | "2300" => Some(HistoryGame::Lotto740),
        "tic-tac-toe" | "tictactoe" | "2400" => Some(HistoryGame::TicTacToe),
        "638" | "2500" => Some(HistoryGame::Lotto638),
        _ => None,
    }
}

fn game_display_name(game: HistoryGame) -> &'static str {
    match game {
        HistoryGame::SuperLotto638 => "SuperLotto638",
        HistoryGame::Lotto649 => "Lotto649",
        HistoryGame::Daily539 => "Daily539",
        HistoryGame::Lotto3D => "Lotto3D",
        HistoryGame::Lotto4D => "Lotto4D",
        HistoryGame::Lotto49M6 => "Lotto49M6",
        HistoryGame::Lotto39M5 => "Lotto39M5",
        HistoryGame::Lotto38M6 => "Lotto38M6",
        HistoryGame::Lotto1224 => "Lotto1224",
        HistoryGame::Lotto740 => "Lotto740",
        HistoryGame::TicTacToe => "TicTacToe",
        HistoryGame::Lotto638 => "Lotto638",
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args.first().cloned().unwrap_or_else(|| "draw".to_string());

    if args.len() < 2 {
        print_usage(&program);
        std::process::exit(2);
    }

    let Some(game) = parse_game(&args[1]) else {
        eprintln!("Invalid game: {}", args[1]);
        print_usage(&program);
        std::process::exit(2);
    };

    let count: usize = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);

    let game_name = game_display_name(game);

    if count == 1 {
        let result = draw_by_game(game);
        println!("🎰 {} Draw Result", game_name);
        println!("  Numbers: {:?}", result.base.numbers);
        if let Some(bonus) = result.bonus {
            println!("  Bonus: {}", bonus);
        }
    } else {
        println!("🎰 {} Draw Results ({} times)", game_name, count);
        println!();
        for i in 1..=count {
            let result = draw_by_game(game);
            print!("  Draw {:2}: Numbers {:?}", i, result.base.numbers);
            if let Some(bonus) = result.bonus {
                print!(" | Bonus {}", bonus);
            }
            println!();
        }
    }
}
