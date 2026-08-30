use crate::time::Instant;
use chess_core::{Color, Move, Sq};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::eval::{EVAL_NONE, INFINITY};
use crate::transposition::{TTEntry, TTFlag, TranspositionTable};
use crate::{board::Board, eval::eval_board, move_gen::MoveGenerator};

const MAX_PLY: u16 = 64;
const MAX_KILLER_MOVES: usize = 2;
const MAX_HISTORY: i32 = 10_000;

pub struct HistoryTable([[[i16; Sq::NB]; Sq::NB]; Color::NB]);

impl Default for HistoryTable {
    fn default() -> Self {
        Self([[[0; Sq::NB]; Sq::NB]; Color::NB])
    }
}

impl HistoryTable {
    #[inline(always)]
    pub fn get(&self, side: Color, from: Sq, to: Sq) -> i16 {
        self.0[side as usize][from as usize][to as usize]
    }

    /// Gravity update formula: naturally bounds values in [-MAX_HISTORY, MAX_HISTORY]
    /// without ever overflowing i16 or requiring periodic resets.
    #[inline(always)]
    pub fn update(&mut self, side: Color, from: Sq, to: Sq, depth: u8) {
        let bonus = depth as i32 * depth as i32;

        let entry = &mut self.0[side as usize][from as usize][to as usize];
        let current = *entry as i32;

        // Gravity formula: current + bonus - (current * |bonus| / MAX_HISTORY)
        let clamped_bonus = bonus.clamp(-MAX_HISTORY, MAX_HISTORY);
        let new_val = current + clamped_bonus - (current * clamped_bonus.abs() / MAX_HISTORY);

        *entry = new_val as i16;
    }

    pub fn clear(&mut self) {
        self.0 = [[[0; Sq::NB]; Sq::NB]; Color::NB];
    }
}

pub type KillerMoves = [Move; MAX_KILLER_MOVES];
type KillerMovesArray = [KillerMoves; MAX_PLY as usize];

#[repr(C)] // to guarantee order
struct Searcher<'a> {
    board: Board,
    tt: &'a TranspositionTable,
    nodes_searched: u64,
    root_ply: u16,
    killer_moves: KillerMovesArray,
    history: HistoryTable,
    pv_table: [[Move; MAX_PLY as usize]; MAX_PLY as usize],
    pv_length: [u16; MAX_PLY as usize],
    stop_requested: Arc<AtomicBool>,
}

impl<'a> Searcher<'a> {
    fn new(board: Board, stop_requested: Arc<AtomicBool>, tt: &'a TranspositionTable) -> Self {
        let root_ply = board.ply;
        Self {
            board,
            stop_requested,
            tt,
            nodes_searched: 0,
            killer_moves: [[Move::default(); MAX_KILLER_MOVES]; MAX_PLY as usize],
            history: HistoryTable::default(),
            root_ply,
            pv_table: [[Move::default(); MAX_PLY as usize]; MAX_PLY as usize],
            pv_length: [0; MAX_PLY as usize],
        }
    }

    #[inline(always)]
    fn ply(&self) -> u16 {
        self.board.ply - self.root_ply
    }

    #[inline(always)]
    fn get_killer_moves(&self) -> KillerMoves {
        let ply = self.ply() as usize;
        debug_assert!(ply < MAX_PLY as usize);
        unsafe { *self.killer_moves.get_unchecked(ply) }
    }

    fn set_killer_move(&mut self, current_move: Move) {
        let ply = self.ply() as usize;
        if ply >= MAX_PLY as usize {
            return;
        }

        let [first_killer, second_killer] = &mut self.killer_moves[ply];
        if *first_killer == current_move {
            return;
        }

        *second_killer = *first_killer;
        *first_killer = current_move;
    }

    #[inline(always)]
    fn store_tt(&self, mov: Move, score: i16, depth: u8, flag: TTFlag) {
        let entry = TTEntry::new(mov, score, EVAL_NONE, depth, flag);
        self.tt.store(self.board.hash, entry, self.ply());
    }

    #[inline(always)]
    fn update_pv(&mut self, ply: u16, mov: Move) {
        let ply = ply as usize;
        if ply >= MAX_PLY as usize - 1 {
            return;
        }
        self.pv_table[ply][0] = mov;
        let copy_len = self.pv_length[ply + 1].min(MAX_PLY - 1) as usize;
        let (current, rest) = self.pv_table[ply..].split_at_mut(1);
        current[0][1..1 + copy_len].copy_from_slice(&rest[0][..copy_len]);
        self.pv_length[ply] = copy_len as u16 + 1;
    }

    fn nega_max(&mut self, mut alpha: i16, beta: i16, depth: u8) -> i16 {
        let ply = self.ply();
        self.pv_length[ply as usize] = 0;
        self.nodes_searched += 1;

        if self.board.ply > 0 && self.board.is_draw(ply) {
            return 0;
        }
        if ply >= MAX_PLY - 1 {
            return eval_board(&self.board);
        }

        let mut tt_move = Move::NONE;
        if let Some(entry) = self.tt.probe(self.board.hash, ply) {
            tt_move = entry.mov;
            if entry.depth >= depth {
                match entry.flag() {
                    TTFlag::Exact => return entry.value,
                    TTFlag::LowerBound if entry.value >= beta => return entry.value,
                    TTFlag::UpperBound if entry.value <= alpha => return entry.value,
                    _ => {}
                }
            }
        }

        if depth == 0 {
            return self.qsearch(alpha, beta);
        }

        let orig_alpha = alpha;
        let mut moves = MoveGenerator::new(tt_move);
        let mut legal_moves = 0;

        let mut best_score = -INFINITY;
        let mut best_move = Move::NONE;

        while let Some(mov) = moves.next(&self.board, self.get_killer_moves(), &self.history) {
            if !self.board.legal(mov) {
                continue;
            }
            legal_moves += 1;

            let undo = self.board.make_move(mov);
            let score = -self.nega_max(-beta, -alpha, depth - 1);
            self.board.undo_move(mov, undo);
            if score > best_score {
                best_score = score;
                if score > alpha {
                    alpha = score;
                    best_move = mov;
                    self.update_pv(ply, mov);
                }
            }

            if score >= beta {
                if !mov.is_capture() {
                    self.set_killer_move(mov);

                    self.history
                        .update(self.board.to_play, mov.from(), mov.to(), depth);

                    // TODO: Maybe penalize previous quiet moves that failed to cause a cutoff
                }

                self.store_tt(mov, best_score, depth, TTFlag::LowerBound);
                return best_score;
            }
        }

        if legal_moves == 0 {
            let score = if self.board.checkers != 0 {
                -INFINITY + ply as i16
            } else {
                0 // Stalemate
            };
            self.store_tt(Move::NONE, score, depth, TTFlag::Exact);
            return score;
        }

        let (flag, mov) = if best_score <= orig_alpha {
            (TTFlag::UpperBound, Move::NONE)
        } else {
            (TTFlag::Exact, best_move)
        };
        self.store_tt(mov, best_score, depth, flag);

        best_score
    }

    fn qsearch(&mut self, mut alpha: i16, beta: i16) -> i16 {
        self.nodes_searched += 1;

        let tt_move = if let Some(entry) = self.tt.probe(self.board.hash, self.ply()) {
            match entry.flag() {
                TTFlag::Exact => return entry.value,
                TTFlag::LowerBound if entry.value >= beta => return entry.value,
                TTFlag::UpperBound if entry.value <= alpha => return entry.value,
                _ => {}
            }
            entry.mov
        } else {
            Move::NONE
        };

        let in_check = self.board.checkers != 0;

        // Stand-pat: not available while in check, since every evasion must be considered.
        let mut best_score = if in_check {
            -INFINITY + self.ply() as i16
        } else {
            let static_eval = eval_board(&self.board);
            if static_eval >= beta {
                return static_eval;
            }
            if static_eval > alpha {
                alpha = static_eval;
            }
            static_eval
        };

        let mut moves = MoveGenerator::quiescence(tt_move);

        while let Some(mov) = moves.next(&self.board, self.get_killer_moves(), &self.history) {
            if !in_check && (!mov.is_capture() && !mov.is_promotion()) {
                continue;
            }
            if !self.board.legal(mov) {
                continue;
            }

            let undo = self.board.make_move(mov);
            let score = -self.qsearch(-beta, -alpha);
            self.board.undo_move(mov, undo);

            if score > best_score {
                best_score = score;
                if score > alpha {
                    alpha = score;
                }
            }

            if score >= beta {
                return best_score;
            }
        }

        // In check with no legal moves is checkmate; best_score is still -INFINITY here.
        best_score
    }

    fn uci_info(&mut self, depth: u8, score: i16, start_time: Instant) -> String {
        let pv_str = self.pv_table[0][..self.pv_length[0] as usize]
            .iter()
            .map(|&m| crate::uci::format_move(m))
            .collect::<Vec<_>>()
            .join(" ");

        let elapsed = start_time.elapsed().as_millis().max(1);
        let nps = (self.nodes_searched as u128 * 1000 / elapsed) as u64;
        format!(
            "info depth {depth} score {} time {elapsed} nps {nps} nodes {} hashfull {} pv {pv_str}",
            crate::uci::format_score(score),
            self.nodes_searched,
            self.tt.hashfull(),
        )
    }
}

pub fn search(
    board: Board,
    max_depth: u8,
    stop_requested: Arc<AtomicBool>,
    tt: &TranspositionTable,
    mut on_info: impl FnMut(String),
) -> Move {
    let mut search = Searcher::new(board, stop_requested, tt);
    let start_time = Instant::now();
    let mut overall_best_move = Move::NONE;

    for current_depth in 1..=max_depth {
        let mut best_move = None;
        let mut best_score = -INFINITY;
        let mut moves = MoveGenerator::new(overall_best_move);

        let mut alpha = -INFINITY;
        let beta = INFINITY;

        while let Some(mov) = moves.next(&search.board, search.get_killer_moves(), &search.history)
        {
            if !search.board.legal(mov) {
                continue;
            }

            let undo = search.board.make_move(mov);
            let score = -search.nega_max(-beta, -alpha, current_depth - 1);
            search.board.undo_move(mov, undo);
            if score > best_score {
                best_score = score;
                best_move = Some(mov);
                if score > alpha {
                    alpha = score;
                    search.update_pv(0, mov);
                }
            }
        }

        if let Some(mov) = best_move {
            overall_best_move = mov;
            search.store_tt(mov, best_score, current_depth, TTFlag::Exact);
        }

        if search.stop_requested.load(Ordering::Relaxed) && !overall_best_move.is_none() {
            break;
        }

        let line = search.uci_info(current_depth, best_score, start_time);
        on_info(line);
    }

    overall_best_move
}
