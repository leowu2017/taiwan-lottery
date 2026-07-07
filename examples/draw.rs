use taiwan_lottery::{draw_by_game, LotteryGame};

fn print_usage(program: &str) {
    eprintln!("Usage:");
    eprintln!("  {program} <game>");
    eprintln!("  {program} <game> <count>");
    eprintln!();
    eprintln!(
            "game: super-lotto638 | lotto649 | daily539 | 3d | 4d | 49m6 | 39m5 | 38m6 | 1224 | 740 | tic-tac-toe | 638 | bingo-bingo"
        );
    eprintln!("count: number of times to draw (default: 1)");
}

fn parse_game(value: &str) -> Option<LotteryGame> {
    LotteryGame::parse(value)
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

    let game_name = format!("{game:?}");

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
