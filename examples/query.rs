use std::path::PathBuf;

use taiwan_lottery::{
    query_history_draw, query_history_draw_from_taiwan_lottory, HistoryDrawQuery, HistoryGame,
    HistorySession,
};

fn print_usage(program: &str) {
    eprintln!("Usage:");
    eprintln!("  {program} local <game> period <PERIOD> [output_dir] [session]");
    eprintln!("  {program} local <game> month <YYYY-MM> [output_dir] [session]");
    eprintln!("  {program} remote <game> period <PERIOD> [session]");
    eprintln!("  {program} remote <game> month <YYYY-MM> [session]");
    eprintln!("  game: super-lotto638 | lotto649 | daily539 | 3d | 4d | 49m6 | 39m5 | 38m6 | 1224 | 740 | tic-tac-toe | 638");
    eprintln!("  session: third | fourth | fifth (default: fifth)");
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

fn parse_session(value: Option<&String>) -> HistorySession {
    match value.map(|v| v.to_ascii_lowercase()) {
        Some(v) if v == "third" => HistorySession::Third,
        Some(v) if v == "fourth" => HistorySession::Fourth,
        _ => HistorySession::Fifth,
    }
}

fn parse_query(mode: &str, value: &str, session: HistorySession) -> HistoryDrawQuery {
    let mut query = match mode {
        "period" => HistoryDrawQuery::by_period(value.to_string()),
        _ => HistoryDrawQuery::by_month(value.to_string()),
    };
    query.session = session;
    query
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
    if query_mode != "period" && query_mode != "month" {
        eprintln!("Query mode must be period or month");
        print_usage(&program);
        std::process::exit(2);
    }

    let query_value = args[4].clone();

    let result = match source {
        "local" => {
            let output_dir = args
                .get(5)
                .map(PathBuf::from)
                .unwrap_or_else(default_output_dir);
            let session = parse_session(args.get(6));
            let query = parse_query(query_mode, &query_value, session);
            query_history_draw(output_dir, game, query)
        }
        "remote" => {
            let session = parse_session(args.get(5));
            let query = parse_query(query_mode, &query_value, session);
            query_history_draw_from_taiwan_lottory(game, query)
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
                println!("lottery_date={}", item.lottery_date.as_deref().unwrap_or(""));
                match item.draw_number_appear {
                    Some(numbers) => println!("draw_number_appear={:?}", numbers),
                    None => println!("draw_number_appear=<not available in local data>"),
                }
                println!("draw_number_size={:?}", item.draw_number_size);
                println!();
            }
        }
        Err(err) => {
            eprintln!("query failed: {err:?}");
            std::process::exit(1);
        }
    }
}
