use std::path::PathBuf;

use taiwan_lottery::{
    query_history_draw, query_history_draw_from_taiwan_lottery, HistoryDrawQuery, HistoryGame,
};

fn print_usage(program: &str) {
    eprintln!("Usage:");
    eprintln!("  {program} local <game> period <PERIOD> [output_dir]");
    eprintln!("  {program} local <game> month <YYYY-MM> [output_dir]");
    eprintln!("  {program} local <game> month-range <YYYY-MM> <YYYY-MM> [output_dir]");
    eprintln!("  {program} remote <game> period <PERIOD>");
    eprintln!("  {program} remote <game> month <YYYY-MM>");
    eprintln!("  {program} remote <game> month-range <YYYY-MM> <YYYY-MM>");
    eprintln!("  game: super-lotto638 | lotto649 | daily539 | 3d | 4d | 49m6 | 39m5 | 38m6 | 1224 | 740 | tic-tac-toe | 638");
}

fn default_output_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("data")
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


fn parse_query(mode: &str, args: &[String], value_index: usize) -> Option<HistoryDrawQuery> {
    let value = args.get(value_index)?.clone();
    let query = match mode {
        "period" => HistoryDrawQuery::by_period(value),
        "month-range" => {
            let end_month = args.get(value_index + 1).cloned().unwrap_or_else(|| value.clone());
            HistoryDrawQuery::by_month_range(value, end_month)
        }
        _ => HistoryDrawQuery::by_month(value),
    };
    Some(query)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let program = args.first().cloned().unwrap_or_else(|| "query".to_string());

    if args.len() < 5 {
        print_usage(&program);
        std::process::exit(2);
    }

    let source = args[1].as_str();
    let Some(game) = parse_game(&args[2]) else {
        eprintln!("Invalid game: {}", args[2]);
        print_usage(&program);
        std::process::exit(2);
    };

    let query_mode = args[3].as_str();
    if query_mode != "period" && query_mode != "month" && query_mode != "month-range" {
        eprintln!("Query mode must be period, month, or month-range");
        print_usage(&program);
        std::process::exit(2);
    }

    // month-range takes two values: <start> <end>, others take one
    let value_count = if query_mode == "month-range" { 2 } else { 1 };

    let result = match source {
        "local" => {
            if args.len() < 4 + value_count {
                eprintln!("Missing query value(s) for mode: {query_mode}");
                print_usage(&program);
                std::process::exit(2);
            }

            let output_dir = args
                .get(4 + value_count)
                .map(PathBuf::from)
                .unwrap_or_else(default_output_dir);
            let Some(query) = parse_query(query_mode, &args, 4) else {
                eprintln!("Missing query value");
                print_usage(&program);
                std::process::exit(2);
            };
            query_history_draw(output_dir, game, query)
        }
        "remote" => {
            if args.len() < 4 + value_count {
                eprintln!("Missing query value(s) for mode: {query_mode}");
                print_usage(&program);
                std::process::exit(2);
            }

            let Some(query) = parse_query(query_mode, &args, 4) else {
                eprintln!("Missing query value");
                print_usage(&program);
                std::process::exit(2);
            };
            query_history_draw_from_taiwan_lottery(game, query)
        }
        _ => {
            print_usage(&program);
            std::process::exit(2);
        }
    };

    match result {
        Ok(page) => {
            println!("total_size={}", page.total_size);
            for item in page.items {
                println!("period={}", item.period);
                println!("date={}", item.date.as_deref().unwrap_or(""));
                println!("numbers={:?}", item.numbers.base.numbers);
                match item.numbers.sorted {
                    Some(numbers_sorted) => println!("numbers_sorted={:?}", numbers_sorted),
                    None => println!("numbers_sorted=<not available>"),
                }
                println!();
            }
        }
        Err(err) => {
            eprintln!("query failed: {err:?}");
            std::process::exit(1);
        }
    }
}
