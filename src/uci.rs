use std::io::{self, BufRead};

use uci_parser::UciCommand;

use crate::board::Board;

pub struct UciState {
    board: Board,
}

impl Default for UciState {
    fn default() -> Self {
        Self {
            board: Board::start_pos(),
        }
    }
}

impl UciState {
    pub fn uci_loop(&mut self) -> bool {
        let mut input_string = String::new();
        io::stdin().lock().read_line(&mut input_string).unwrap();

        let command = match input_string.parse::<UciCommand>() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to parse: {e}");
                return true;
            }
        };

        match command {
            UciCommand::Uci => {
                println!("id name lucky_chess 1.0\nid author Razvan\nuciok");
            }
            UciCommand::Debug(_) => todo!(),
            UciCommand::IsReady => {
                println!("readyok")
            }
            UciCommand::SetOption { name, value } => todo!(),
            UciCommand::Register { name, code } => todo!(),
            UciCommand::UciNewGame => {
                self.board = Board::start_pos();
            }
            UciCommand::Position { fen, moves } => {
                if let Some(fen) = fen {
                    let Some(new_board) = Board::from_fen(&fen) else {
                        eprintln!("Invalid FEN");
                        return true;
                    };

                    self.board = new_board;
                }
            }
            UciCommand::Go(uci_search_options) => todo!(),
            UciCommand::Stop => todo!(),
            UciCommand::PonderHit => todo!(),
            UciCommand::Quit => return false,
        }

        true
    }
}
