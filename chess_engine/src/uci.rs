use chess_core::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use uci_parser::UciCommand;

use crate::eval::INFINITY;
use crate::nnue::Accumulator;
use crate::time::TimeManager;
use crate::transposition::TranspositionTable;
use crate::{board::Board, move_gen::gen_all_moves, search::search};

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

        // Custom developer commands
        if trimmed.eq_ignore_ascii_case("d") || trimmed.eq_ignore_ascii_case("display") {
            self.display_board();
            return true;
        }

        if trimmed.eq_ignore_ascii_case("eval") {
            let acc = crate::nnue::Accumulator::from_board(&self.board);
            let eval = acc.eval(self.board.to_play);
            self.output_line(format!("score: {}", format_score(eval)));
            return true;
        }

        if let Some(rest) = trimmed.strip_prefix("perft") {
            let depth = rest.trim().parse::<u8>().unwrap_or(1).max(1);
            self.run_perft(depth);
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
                #[cfg(not(target_family = "wasm"))]
                if let Some(thread) = self.search_thread.take() {
                    let _ = thread.join();
                }
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
                #[cfg(not(target_family = "wasm"))]
                if let Some(thread) = self.search_thread.take() {
                    self.stop_requested.store(true, Ordering::Relaxed);
                    let _ = thread.join();
                }
                self.stop_requested.store(false, Ordering::Relaxed);
                self.board = Board::start_pos();
                self.tt.clear();
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

                let mut next_acc = Accumulator::default();
                for uci_move in moves {
                    if let Some(mov) = self.find_move(uci_move) {
                        self.board.make_move(mov, &mut next_acc);
                    } else {
                        eprintln!("Illegal or unrecognized move in position command");
                        break;
                    }
                }
            }
            UciCommand::Go(opts) => {
                if let Some(depth) = opts.perft {
                    self.run_perft(depth as u8);
                } else {
                    let time_manager = TimeManager::from_uci_options(&opts, self.board.to_play);
                    self.start_search(time_manager);
                }
            }
            UciCommand::Stop => {
                self.stop();
                #[cfg(not(target_family = "wasm"))]
                if let Some(thread) = self.search_thread.take() {
                    let _ = thread.join();
                }
            }
            UciCommand::PonderHit => {}
            UciCommand::Quit => {
                self.stop();
                #[cfg(not(target_family = "wasm"))]
                if let Some(thread) = self.search_thread.take() {
                    let _ = thread.join();
                }
                return false;
            }
        }

        true
    }

    fn display_board(&self) {
        let mut out = format!("{:?}\n", self.board);
        out.push_str(&format!("FEN: {}\n", self.board.to_fen()));
        out.push_str(&format!("Key: 0x{:016X}", self.board.hash));
        self.output_line(out);
    }

    fn run_perft(&mut self, depth: u8) {
        #[cfg(not(target_family = "wasm"))]
        if let Some(thread) = self.search_thread.take() {
            self.stop_requested.store(true, Ordering::Relaxed);
            let _ = thread.join();
        }

        let mut board = self.board.clone();
        let nodes = crate::perft::perft(&mut board, depth);
        self.output_line(format!("Nodes searched: {nodes}"));
    }

    fn start_search(&mut self, time_manager: TimeManager) {
        #[cfg(not(target_family = "wasm"))]
        if let Some(thread) = self.search_thread.take() {
            self.stop_requested.store(true, Ordering::Relaxed);
            let _ = thread.join();
        }

        self.stop_requested.store(false, Ordering::Relaxed);

        if let Some(tt) = Arc::get_mut(&mut self.tt) {
            tt.new_search();
        }

        let board = self.board.clone();
        let stop_requested = self.stop_requested.clone();
        let tt = self.tt.clone();
        let output_cb = self.output_cb.clone();

        let run_search = move || {
            let on_info = |line: String| {
                output_cb(line);
            };

            let best = search(board.clone(), time_manager, stop_requested, &tt, on_info);
            let mut next_acc = Accumulator::default();
            let mut ponder = None;
            if best != Move::NONE {
                let mut next_board = board;
                next_board.make_move(best, &mut next_acc);
                if let Some(entry) = tt.probe(next_board.hash, 1)
                    && entry.mov != Move::NONE
                    && next_board.legal(entry.mov)
                {
                    ponder = Some(entry.mov);
                }
            }

            let best_line = match ponder {
                Some(p) => format!("bestmove {} ponder {}", format_move(best), format_move(p)),
                None => format!("bestmove {}", format_move(best)),
            };
            output_cb(best_line);
        };

        #[cfg(not(target_family = "wasm"))]
        {
            self.search_thread = Some(std::thread::spawn(run_search));
        }

        #[cfg(target_family = "wasm")]
        {
            run_search();
        }
    }

    fn find_move(&mut self, uci_move: uci_parser::types::UciMove) -> Option<Move> {
        let from_sq = Sq::new(uci_move.src.0 as u8, uci_move.src.1 as u8)?;
        let to_sq = Sq::new(uci_move.dst.0 as u8, uci_move.dst.1 as u8)?;

        for &scored_move in gen_all_moves(&self.board).as_slice() {
            let mov = scored_move.mov;
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
    use std::sync::Mutex;

    #[test]
    fn test_process_command_basic() {
        let mut uci = UciState::new(|_| {});
        assert!(uci.process_command("uci"));
        assert!(uci.process_command("isready"));
        assert!(uci.process_command("position startpos moves e2e4 e7e5"));
        assert!(!uci.process_command("quit"));
    }

    #[test]
    fn test_process_command_display_and_eval() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let out_clone = output.clone();
        let mut uci = UciState::new(move |line| {
            out_clone.lock().unwrap().push(line);
        });

        assert!(uci.process_command("d"));
        assert!(uci.process_command("eval"));

        let lines = output.lock().unwrap().clone();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("Side to move: White"));
        assert!(lines[0].contains("FEN: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"));
        assert!(lines[1].starts_with("score: cp"));
    }

    #[test]
    fn test_process_command_perft() {
        let output = Arc::new(Mutex::new(Vec::new()));
        let out_clone = output.clone();
        let mut uci = UciState::new(move |line| {
            out_clone.lock().unwrap().push(line);
        });

        assert!(uci.process_command("go perft 1"));
        let lines = output.lock().unwrap().clone();
        assert!(lines.iter().any(|l| l.contains("Nodes searched: 20")));
    }

    #[test]
    fn test_process_command_ucinewgame_clears_tt() {
        let mut uci = UciState::new(|_| {});
        // Store entry in TT
        let entry = crate::transposition::TTEntry::new(
            Move::NONE,
            100,
            50,
            4,
            crate::transposition::TTFlag::Exact,
        );
        uci.tt.store(uci.board.hash, entry, 0);
        assert!(uci.tt.probe(uci.board.hash, 0).is_some());

        assert!(uci.process_command("ucinewgame"));
        assert!(uci.tt.probe(uci.board.hash, 0).is_none());
    }
}
