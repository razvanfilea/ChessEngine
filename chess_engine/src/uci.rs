use chess_core::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use uci_parser::UciCommand;

use crate::eval::INFINITY;
use crate::transposition::TranspositionTable;
use crate::{board::Board, move_gen::MoveGenerator, search::search};

pub type OutputCallback = Arc<dyn Fn(String) + Send + Sync>;

pub struct UciState {
    board: Board,
    #[cfg(not(target_family = "wasm"))]
    search_thread: Option<std::thread::JoinHandle<()>>,
    stop_requested: Arc<AtomicBool>,
    tt: Arc<TranspositionTable>,
    output_cb: OutputCallback,
}

impl Default for UciState {
    fn default() -> Self {
        Self::new(|line| println!("{line}"))
    }
}

impl UciState {
    pub fn new(output_cb: impl Fn(String) + Send + Sync + 'static) -> Self {
        let default_tt_mb = if cfg!(miri) { 1 } else { 64 };
        Self {
            board: Board::start_pos(),
            #[cfg(not(target_family = "wasm"))]
            search_thread: None,
            stop_requested: Arc::default(),
            tt: Arc::new(TranspositionTable::new(default_tt_mb)),
            output_cb: Arc::new(output_cb),
        }
    }

    #[inline(always)]
    pub fn output_line(&self, line: impl Into<String>) {
        (self.output_cb)(line.into());
    }

    pub fn stop(&mut self) {
        self.stop_requested.store(true, Ordering::Relaxed);
    }

    pub fn process_command(&mut self, input_string: &str) -> bool {
        let trimmed = input_string.trim();
        if trimmed.is_empty() {
            return true;
        }

        let command = match trimmed.parse::<UciCommand>() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to parse: {e}");
                return true;
            }
        };

        match command {
            UciCommand::Uci => {
                self.output_line(
                    r#"id name lucky_chess 1.0
id author Razvan
option name Hash type spin default 64 min 1 max 1024
option name ClearHash type button
uciok"#,
                );
            }
            UciCommand::Debug(_) => {}
            UciCommand::IsReady => {
                self.output_line("readyok");
            }
            UciCommand::SetOption { name, value } => {
                if name.eq_ignore_ascii_case("Hash") {
                    if let Some(mb) = value.and_then(|v| v.parse().ok()) {
                        self.tt = Arc::new(TranspositionTable::new(mb));
                    }
                } else if name.eq_ignore_ascii_case("ClearHash") {
                    self.tt.clear();
                }
            }
            UciCommand::Register { .. } => self.output_line("registration ok"),
            UciCommand::UciNewGame => {
                self.board = Board::start_pos();
            }
            UciCommand::Position { fen, moves } => {
                self.board = if let Some(fen) = fen {
                    let Some(new_board) = Board::from_fen(&fen) else {
                        eprintln!("Invalid FEN");
                        return true;
                    };
                    new_board
                } else {
                    Board::start_pos()
                };

                for uci_move in moves {
                    if let Some(mov) = self.find_move(uci_move) {
                        self.board.make_move(mov);
                    } else {
                        eprintln!("Illegal or unrecognized move in position command");
                        break;
                    }
                }
            }
            UciCommand::Go(opts) => {
                let depth = opts.depth.unwrap_or(12) as u8;
                self.start_search(depth);
            }
            UciCommand::Stop => {
                self.stop();
            }
            UciCommand::PonderHit => {}
            UciCommand::Quit => return false,
        }

        true
    }

    fn start_search(&mut self, depth: u8) {
        let board = self.board.clone();
        let stop_requested = self.stop_requested.clone();
        stop_requested.store(false, Ordering::Relaxed);

        if let Some(tt) = Arc::get_mut(&mut self.tt) {
            tt.new_search();
        }
        let tt = self.tt.clone();
        let output_cb = self.output_cb.clone();

        let run_search = move || {
            let on_info = |line: String| {
                output_cb(line);
            };

            let best = search(board, depth, stop_requested, &tt, on_info);
            let best_line = format!("bestmove {}", format_move(best));
            output_cb(best_line);
        };

        #[cfg(not(target_family = "wasm"))]
        {
            if let Some(thread) = self.search_thread.take() {
                let _ = thread.join();
            }
            self.search_thread = Some(std::thread::spawn(run_search));
        }

        #[cfg(target_family = "wasm")]
        {
            run_search();
        }
    }

    fn find_move(&mut self, uci_move: uci_parser::types::UciMove) -> Option<Move> {
        let mut generator = MoveGenerator::default();
        let from_sq = Sq::new(uci_move.src.0 as u8, uci_move.src.1 as u8)?;
        let to_sq = Sq::new(uci_move.dst.0 as u8, uci_move.dst.1 as u8)?;

        while let Some(mov) = generator.next(&self.board, [Move::default(); 2]) {
            if mov.from() == from_sq && mov.to() == to_sq && self.board.legal(mov) {
                // Check promotion match if applicable
                if let Some(target_promo) = uci_move.promote {
                    let promo_piece = match target_promo {
                        uci_parser::types::Piece::Queen => Piece::Queen,
                        uci_parser::types::Piece::Rook => Piece::Rook,
                        uci_parser::types::Piece::Bishop => Piece::Bishop,
                        uci_parser::types::Piece::Knight => Piece::Knight,
                        _ => return None,
                    };
                    if mov.promotion_piece() == Some(promo_piece) {
                        return Some(mov);
                    }
                } else if !mov.is_promotion() {
                    return Some(mov);
                }
            }
        }
        None
    }
}

pub fn format_move(mov: Move) -> String {
    let promo = match mov.promotion_piece() {
        Some(Piece::Queen) => "q",
        Some(Piece::Rook) => "r",
        Some(Piece::Bishop) => "b",
        Some(Piece::Knight) => "n",
        _ => "",
    };
    format!("{}{}{promo}", mov.from(), mov.to())
}

pub fn format_score(score: i16) -> String {
    const MATE_THRESHOLD: i16 = 29_000;

    if score > MATE_THRESHOLD {
        // We are mating the opponent
        let plies_to_mate = INFINITY - score;
        let moves_to_mate = (plies_to_mate + 1) / 2;
        format!("mate {moves_to_mate}")
    } else if score < -MATE_THRESHOLD {
        // Opponent is mating us
        let plies_to_mate = INFINITY + score;
        let moves_to_mate = (plies_to_mate + 1) / 2;
        format!("mate -{moves_to_mate}")
    } else {
        format!("cp {score}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_command_basic() {
        let mut uci = UciState::new(|_| {});
        assert!(uci.process_command("uci"));
        assert!(uci.process_command("isready"));
        assert!(uci.process_command("position startpos moves e2e4 e7e5"));
        assert!(!uci.process_command("quit"));
    }
}
