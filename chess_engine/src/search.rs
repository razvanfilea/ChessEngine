use crate::time::{Instant, TimeManager};
use chess_core::bitboard::{RANK_2, RANK_7};
use chess_core::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::eval::{self, EVAL_NONE, INFINITY, MATE_THRESHOLD};
use crate::transposition::{TTEntry, TTFlag, TranspositionTable};
use crate::{board::Board, eval::eval_board, move_gen::MoveGenerator};

const MAX_PLY: u16 = 64;
const MAX_KILLER_MOVES: usize = 2;
const MAX_HISTORY: i32 = 10_000;

const NULL_MOVE_REDUCTION: u8 = 3;
const FUTILITY_MARGIN: i16 = 100;
const FUTILITY_MAX_DEPTH: u8 = 8;
const RFP_MARGIN: i16 = 100;
const RFP_DEPTH: u8 = 5;

const DELTA_MARGIN: i16 = 200;
const GLOBAL_DELTA_MARGIN: i16 = 900; // Queen

const ASPIRATION_INITIAL_DELTA: i16 = 20;
const ASPIRATION_FLUCTUATION: i16 = 100;
const ASPIRATION_MIN_DEPTH: u8 = 5;

static LMR_TABLE: std::sync::LazyLock<LmrTable> = std::sync::LazyLock::new(|| {
    let mut table = [[(0, 0); MAX_PLY as usize]; MAX_PLY as usize];

    let mut depth = 1;
    while depth < MAX_PLY {
        let mut moves = 1;
        while moves < MAX_PLY {
            let r = (0.75 + (depth as f64).ln() * (moves as f64).ln() / 2.25) as i8;
            table[depth as usize][moves as usize] = (r, (r.max(1) - 1) as u8);
            moves += 1;
        }

        depth += 1;
    }

    table
});

type LmrTable = [[(i8, u8); MAX_PLY as usize]; MAX_PLY as usize];

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
    lmr_table: &'static LmrTable,
    nodes_searched: u64,
    root_ply: u16,
    killer_moves: KillerMovesArray,
    history: HistoryTable,
    pv_table: [[Move; MAX_PLY as usize]; MAX_PLY as usize],
    pv_length: [u16; MAX_PLY as usize],
    stop_requested: Arc<AtomicBool>,
    time_manager: TimeManager,
    stopped: bool,
}

impl<'a> Searcher<'a> {
    fn new(
        board: Board,
        stop_requested: Arc<AtomicBool>,
        tt: &'a TranspositionTable,
        time_manager: TimeManager,
    ) -> Self {
        let root_ply = board.ply;
        let stopped = stop_requested.load(Ordering::Relaxed);
        Self {
            board,
            stop_requested,
            tt,
            lmr_table: &*LMR_TABLE,
            time_manager,
            stopped,
            nodes_searched: 0,
            killer_moves: [[Move::default(); MAX_KILLER_MOVES]; MAX_PLY as usize],
            history: HistoryTable::default(),
            root_ply,
            pv_table: [[Move::default(); MAX_PLY as usize]; MAX_PLY as usize],
            pv_length: [0; MAX_PLY as usize],
        }
    }

    #[inline(always)]
    fn check_limits(&mut self) {
        if self.nodes_searched & 2047 == 0
            && (self.stop_requested.load(Ordering::Relaxed)
                || self
                    .time_manager
                    .is_hard_limit_exceeded(self.nodes_searched))
        {
            self.stopped = true;
            self.stop_requested.store(true, Ordering::Relaxed);
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
    fn store_tt(&self, mov: Move, score: i16, eval: i16, depth: u8, flag: TTFlag) {
        if self.stopped {
            return;
        }
        let entry = TTEntry::new(mov, score, eval, depth, flag);
        self.tt.store(self.board.hash, entry, self.ply());
    }

    #[inline(always)]
    fn update_pv(&mut self, ply: u16, mov: Move) {
        let ply = ply as usize;
        if ply >= MAX_PLY as usize - 1 {
            return;
        }
        self.pv_table[ply][0] = mov;
        let next_len = (self.pv_length[ply + 1] as usize).min(MAX_PLY as usize - 1 - ply);
        let (current, rest) = self.pv_table[ply..].split_at_mut(1);
        for (dst, &src) in current[0][1..1 + next_len].iter_mut().zip(&rest[0][..next_len]) {
            *dst = src;
        }
        self.pv_length[ply] = (next_len + 1) as u16;
    }

    #[inline(always)]
    fn get_lmr(&self, is_pv: bool, depth: u8, mov_index: u8) {
        LMR_TABLE[depth.max(63) as usize][mov_index.max(63) as usize];
    }

    fn nega_max<const IS_PV: bool>(
        &mut self,
        mut alpha: i16,
        beta: i16,
        depth: u8,
        can_null: bool,
    ) -> i16 {
        let ply = self.ply();
        let in_check = self.board.in_check();
        self.pv_length[ply as usize] = 0;
        self.nodes_searched += 1;
        self.check_limits();

        if self.stopped {
            return 0;
        }

        if ply > 0 && self.board.is_draw() {
            return 0;
        }

        if depth == 0 {
            return self.qsearch(alpha, beta);
        }

        let (tt_move, mut static_eval) = match self.tt.probe(self.board.hash, ply) {
            Some(entry) => {
                if let Some(score) = entry.cutoff(depth, alpha, beta) {
                    if !entry.mov.is_none() && self.board.legal(entry.mov) {
                        self.pv_table[ply as usize][0] = entry.mov;
                        self.pv_length[ply as usize] = 1;
                    }
                    return score;
                }
                (entry.mov, entry.eval)
            }
            None => (Move::NONE, EVAL_NONE),
        };

        if static_eval == EVAL_NONE && !self.board.in_check() {
            static_eval = eval_board(&self.board);
        }

        if ply >= MAX_PLY - 1 {
            return static_eval;
        }

        // Reverse Futility Pruning
        if !IS_PV
            && !in_check
            && depth <= RFP_DEPTH
            && (tt_move.is_none() || !tt_move.is_capture())
            && static_eval >= beta.saturating_add(RFP_MARGIN * depth as i16)
        {
            return ((static_eval as i32 + beta as i32) / 2) as i16;
        }

        // Null Move Pruning
        if !IS_PV
            && !in_check
            && can_null
            && depth >= NULL_MOVE_REDUCTION
            && static_eval >= beta
            && beta < MATE_THRESHOLD
            && self.board.has_non_pawn_material(self.board.to_play)
        {
            let undo = self.board.make_null_move();
            let score =
                -self.nega_max::<true>(-beta, -beta + 1, depth - NULL_MOVE_REDUCTION, false);
            self.board.undo_null_move(undo);

            if self.stopped {
                return 0;
            }

            if score >= beta {
                return beta;
            }
        }

        let futility_margin_eval = static_eval + FUTILITY_MARGIN * depth as i16;
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
            let move_gives_check = self.board.in_check();

            // Futility Pruning
            if !IS_PV
                && depth < FUTILITY_MAX_DEPTH
                && legal_moves > 3
                && !(alpha > MATE_THRESHOLD)
                && !in_check
                && futility_margin_eval <= alpha
                && !mov.is_tactical()
                && mov.flags() != MoveFlags::DoublePawn
                && !move_gives_check
            {
                if static_eval > best_score {
                    best_score = static_eval;
                }
                self.board.undo_move(mov, undo);
                continue;
            }

            // Principal Variation
            let score = if IS_PV && legal_moves > 1 {
                let score = -self.nega_max::<false>(-alpha - 1, -alpha, depth - 1, can_null);

                if score > alpha && score < beta && !self.stopped {
                    // Re-search with full window
                    -self.nega_max::<true>(-beta, -alpha, depth - 1, can_null)
                } else {
                    score
                }
            } else {
                -self.nega_max::<IS_PV>(-beta, -alpha, depth - 1, can_null)
            };
            self.board.undo_move(mov, undo);

            if self.stopped {
                return 0;
            }

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

                self.store_tt(mov, best_score, static_eval, depth, TTFlag::LowerBound);
                return best_score;
            }
        }

        if legal_moves == 0 {
            let score = if in_check {
                -INFINITY + ply as i16
            } else {
                0 // Stalemate
            };
            self.store_tt(Move::NONE, score, static_eval, depth, TTFlag::Exact);
            return score;
        }

        let (flag, mov) = if best_score <= orig_alpha {
            (TTFlag::UpperBound, Move::NONE)
        } else {
            (TTFlag::Exact, best_move)
        };
        self.store_tt(mov, best_score, static_eval, depth, flag);

        best_score
    }

    fn qsearch(&mut self, mut alpha: i16, beta: i16) -> i16 {
        self.nodes_searched += 1;
        self.check_limits();
        if self.stopped {
            return 0;
        }

        let ply = self.ply();
        if ply >= MAX_PLY - 1 {
            return eval_board(&self.board);
        }

        let (tt_move, mut static_eval) = match self.tt.probe(self.board.hash, ply) {
            Some(entry) => {
                if let Some(score) = entry.cutoff(0, alpha, beta) {
                    return score;
                }
                (entry.mov, entry.eval)
            }
            None => (Move::NONE, EVAL_NONE),
        };

        let orig_alpha = alpha;
        let in_check = self.board.in_check();
        // Stand-pat: not available while in check, since every evasion must be considered.
        let mut best_score = if in_check {
            -INFINITY + ply as i16
        } else {
            if static_eval == EVAL_NONE {
                static_eval = eval_board(&self.board);
            }
            if static_eval >= beta {
                return static_eval;
            }
            if static_eval > alpha {
                alpha = static_eval;
            }

            // Global Delta Pruning
            let our_pawns = self.board.color_piece(Piece::Pawn, self.board.to_play);
            let has_promoting_pawns = match self.board.to_play {
                Color::White => (our_pawns & RANK_7) != 0,
                Color::Black => (our_pawns & RANK_2) != 0,
            };
            if !has_promoting_pawns && static_eval < alpha - GLOBAL_DELTA_MARGIN {
                return static_eval;
            }

            static_eval
        };

        let mut moves = MoveGenerator::quiescence(tt_move);
        let mut best_move = Move::NONE;

        while let Some(mov) = moves.next(&self.board, self.get_killer_moves(), &self.history) {
            if !in_check && !mov.is_tactical() {
                continue;
            }
            if !in_check
                && let Some(victim) = self.board.piece_at(mov.to())
                && (static_eval
                    + eval::PIECE_VALUES_MG[victim.piece() as usize] as i16
                    + DELTA_MARGIN)
                    < alpha
            {
                continue;
            }
            if !self.board.legal(mov) {
                continue;
            }

            let undo = self.board.make_move(mov);
            let score = -self.qsearch(-beta, -alpha);
            self.board.undo_move(mov, undo);

            if self.stopped {
                return 0;
            }

            if score > best_score {
                best_score = score;
                best_move = mov;
                if score > alpha {
                    alpha = score;
                }
            }

            if score >= beta {
                self.store_tt(mov, best_score, static_eval, 0, TTFlag::LowerBound);
                return best_score;
            }
        }

        if best_score <= orig_alpha {
            self.store_tt(Move::NONE, best_score, static_eval, 0, TTFlag::UpperBound);
        } else if in_check {
            // We can only store as exact if in check, otherwise we didnt even check all moves
            self.store_tt(best_move, best_score, static_eval, 0, TTFlag::Exact);
        };

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
    time_manager: TimeManager,
    stop_requested: Arc<AtomicBool>,
    tt: &TranspositionTable,
    mut on_info: impl FnMut(String),
) -> Move {
    let start_time = Instant::now();
    let max_depth = time_manager.limits.max_depth;
    let mut search = Searcher::new(board, stop_requested, tt, time_manager);
    let mut best_score = -INFINITY;
    let mut completed_best_move = Move::NONE;
    let mut prev_best_move = Move::NONE;

    'iterative: for current_depth in 1..=max_depth {
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        let mut delta = ASPIRATION_INITIAL_DELTA;

        if current_depth >= ASPIRATION_MIN_DEPTH {
            alpha = best_score.saturating_sub(delta).max(-INFINITY);
            beta = best_score.saturating_add(delta).min(INFINITY);
        }

        'aspiration: loop {
            let score = search.nega_max::<true>(alpha, beta, current_depth, true);
            if search.stopped {
                break 'iterative;
            }

            if score <= alpha && alpha > -INFINITY {
                beta = ((alpha as i32 + beta as i32) / 2) as i16;
                alpha = best_score.saturating_sub(delta).max(-INFINITY);
                if score < -MATE_THRESHOLD {
                    alpha = -INFINITY;
                }
            } else if score >= beta && beta < INFINITY {
                beta = best_score.saturating_add(delta).min(INFINITY);
                if score > MATE_THRESHOLD {
                    beta = INFINITY;
                }
            } else {
                best_score = score;
                break 'aspiration;
            }

            if delta > ASPIRATION_FLUCTUATION {
                alpha = -INFINITY;
                beta = INFINITY;
            } else {
                delta = delta.saturating_add(delta / 2);
            }
        }

        if search.stopped {
            break;
        }

        let current_best_move = search.pv_table[0][0];
        if current_best_move != Move::NONE && search.board.legal(current_best_move) {
            completed_best_move = current_best_move;
        }

        let line = search.uci_info(current_depth, best_score, start_time);
        on_info(line);

        let move_is_stable = current_best_move == prev_best_move;
        prev_best_move = current_best_move;

        if search.stop_requested.load(Ordering::Relaxed)
            || search
                .time_manager
                .should_stop_after_depth(current_depth, move_is_stable)
        {
            search.stopped = true;
            break 'iterative;
        }
    }

    if completed_best_move == Move::NONE || !search.board.legal(completed_best_move) {
        let pv_move = search.pv_table[0][0];
        let tt_move = tt.probe(search.board.hash, 0).map_or(Move::NONE, |e| e.mov);

        completed_best_move = if pv_move != Move::NONE && search.board.legal(pv_move) {
            pv_move
        } else if tt_move != Move::NONE && search.board.legal(tt_move) {
            tt_move
        } else {
            crate::move_gen::gen_all_moves(&search.board)
                .as_slice()
                .iter()
                .copied()
                .map(|scored| scored.mov)
                .find(|&m| search.board.legal(m))
                .unwrap_or(Move::NONE)
        };
    }

    completed_best_move
}
