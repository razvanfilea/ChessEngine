mod bench;
mod perft;

use chess_engine::uci::UciState;
use std::io::{self, BufRead};

fn handle_cli_command(cmd: &str) -> bool {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return false;
    }

    let mut parts = trimmed.split_whitespace();
    let command = parts.next().unwrap_or("");

    if command.eq_ignore_ascii_case("bench") {
        let depth = parts
            .next()
            .and_then(|s| s.parse::<u8>().ok())
            .unwrap_or(10);
        bench::run_bench(depth, 16, |line| println!("{line}"));
        return true;
    }

    if command.eq_ignore_ascii_case("perft") || command.eq_ignore_ascii_case("perft-suite") {
        let depth = parts.next().and_then(|s| s.parse::<u8>().ok());
        perft::run_perft_suite(depth);
        return true;
    }

    false
}

fn main() {
    let mut uci = UciState::new(|line| println!("{line}"));

    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 {
        let cmd = args[1..].join(" ");
        if handle_cli_command(&cmd) {
            return;
        }
        uci.process_command(&cmd);
        return;
    }

    let stdin = io::stdin();
    let mut input_string = String::new();
    while stdin.lock().read_line(&mut input_string).unwrap_or(0) > 0 {
        let trimmed = input_string.trim();

        if trimmed.eq_ignore_ascii_case("bench")
            || trimmed.to_ascii_lowercase().starts_with("bench ")
        {
            let depth = trimmed
                .split_whitespace()
                .nth(1)
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(10);
            bench::run_bench(depth, 16, |line| println!("{line}"));
            input_string.clear();
            continue;
        }

        if trimmed.eq_ignore_ascii_case("perft-suite")
            || trimmed.to_ascii_lowercase().starts_with("perft-suite ")
            || trimmed.to_ascii_lowercase().starts_with("perft suite")
        {
            let depth = trimmed
                .split_whitespace()
                .filter_map(|s| s.parse::<u8>().ok())
                .next();
            perft::run_perft_suite(depth);
            input_string.clear();
            continue;
        }

        if !uci.process_command(&input_string) {
            break;
        }
        input_string.clear();
    }
}
