use chess_engine::uci::UciState;
use std::io::{self, BufRead};

fn main() {
    // lucky_chess::perft::perft_start(7);
    // lucky_chess::perft::perft_kiwipete(5);
    // lucky_chess::perft::perft_pos3(6);
    // lucky_chess::perft::perft_pos4(5);
    // lucky_chess::perft::perft_pos4_mirrored(5);
    // lucky_chess::perft::perft_pos5(5);
    // lucky_chess::perft::perft_pos6(5);

    let mut uci = UciState::new(|line| println!("{line}"));
    let stdin = io::stdin();
    let mut input_string = String::new();
    while stdin.lock().read_line(&mut input_string).unwrap_or(0) > 0 {
        if !uci.process_command(&input_string) {
            break;
        }
        input_string.clear();
    }
}
