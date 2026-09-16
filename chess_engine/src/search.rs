use crate::move_gen::scoring::see_ge;
use crate::move_gen::{MAX_MOVES, MoveListPtr, ScoredMove, gen_all_moves};
use crate::nnue::Accumulator;
use crate::time::{Instant, TimeManager};
use chess_core::bitboard::{RANK_2, RANK_7};
use chess_core::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::transposition::{TTEntry, TTFlag, TranspositionTable};
use crate::{board::Board, move_gen::MoveGenerator};

mod history;
mod params;
mod stack;

pub use history::*;
pub use params::*;
pub use stack::*;

pub fn search(
    board: Board,
    time_manager: TimeManager,
    stop_requested: Arc<AtomicBool>,
    tt: &TranspositionTable,
    mut on_info: impl FnMut(String),
) -> Move {
    let start_time = Instant::now();
    let max_depth = time_manager.limits.max_depth;

    let mut move_buffer = [ScoredMove::default(); MAX_PLY as usize * MAX_MOVES / 2];
    let move_ptr = MoveListPtr(move_buffer.as_mut_ptr());
    let mut search = Searcher::new(board, stop_requested, tt, time_manager);
    let mut best_score = -INFINITY;
    let mut completed_best_move = Move::NONE;
    let mut prev_best_move = Move::NONE;

    'iterative: for current_depth in 1..=max_depth {
        search.selective_depth = 0;
        best_score = search.aspiration_search(move_ptr, current_depth, best_score);

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

    search.resolve_best_move(completed_best_move)
}
#[repr(C)]
struct Searcher<'a> {
    nodes_searched: u64,
    root_ply: u16,
    selective_depth: u8,
    stopped: bool,
    tt: &'a TranspositionTable,
    board: Board,
    stop_requested: Arc<AtomicBool>,
    time_manager: TimeManager,
    lmr_table: &'static LmrTable,
    nnue_accumulator: Box<[Accumulator; MAX_PLY as usize]>,
    stack: SearchStack,
    pv_table: [[Move; MAX_PLY as usize]; MAX_PLY as usize],
    history: HistoryTable,
    cont_history: ContinuationHistoryTable,
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
        let mut nnue_accumulator: Box<[Accumulator; MAX_PLY as usize]> =
            vec![Accumulator::default(); MAX_PLY as usize]
                .into_boxed_slice()
                .try_into()
                .unwrap();
        nnue_accumulator[0] = Accumulator::from_board(&board);

        let stack = SearchStack::new();

        Self {
            board,
            stop_requested,
            tt,
            lmr_table: &*LMR_TABLE,
            time_manager,
            stopped,
            selective_depth: 0,
            nodes_searched: 0,
            nnue_accumulator,
            stack,
            root_ply,
            pv_table: [[Move::default(); MAX_PLY as usize]; MAX_PLY as usize],
            history: HistoryTable::default(),
            cont_history: ContinuationHistoryTable::default(),
        }
    }

    fn aspiration_search(&mut self, move_ptr: MoveListPtr, depth: u8, prev_score: i16) -> i16 {
        let mut alpha = -INFINITY;
        let mut beta = INFINITY;
        let mut delta = ASPIRATION_INITIAL_DELTA;

        if depth >= ASPIRATION_MIN_DEPTH {
            alpha = prev_score.saturating_sub(delta).max(-INFINITY);
            beta = prev_score.saturating_add(delta).min(INFINITY);
        }

        loop {
            let score = self.nega_max::<true>(move_ptr, alpha, beta, depth, true);
            if self.stopped {
                return prev_score;
            }

            if score <= alpha && alpha > -INFINITY {
                beta = ((alpha as i32 + beta as i32) / 2) as i16;
                delta = delta.saturating_add(delta / 2);
                if delta > ASPIRATION_FLUCTUATION || score < -MATE_THRESHOLD {
                    alpha = -INFINITY;
                    beta = INFINITY;
                } else {
                    alpha = score.saturating_sub(delta).max(-INFINITY);
                }
            } else if score >= beta && beta < INFINITY {
                delta = delta.saturating_add(delta / 2);
                if delta > ASPIRATION_FLUCTUATION || score > MATE_THRESHOLD {
                    alpha = -INFINITY;
                    beta = INFINITY;
                } else {
                    beta = score.saturating_add(delta).min(INFINITY);
                }
            } else {
                return score;
            }
        }
    }

    #[inline(always)]
    fn check_limits(&mut self) {
        if self.nodes_searched & TIME_CHECK_MASK == 0
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

    fn update_quiet_history(
        &mut self,
        best: Move,
        tried: &[Move],
        conthist: &ContHistPtrs,
        depth: u8,
    ) {
        let side = self.board.to_play;
        self.history
            .update_bonus(side, best.from(), best.to(), depth);

        for &m in tried {
            self.history.update_malus(side, m.from(), m.to(), depth);
        }

        let curr_piece = unsafe { self.board.piece_type_at(best.from()) };
        let bonus = history_depth_bonus(depth);
        for (i, &ptr) in conthist.iter().enumerate() {
            let scaled = bonus >> i;
            conthist_update(ptr, curr_piece, best.to(), scaled);
        }

        for &m in tried {
            let piece = unsafe { self.board.piece_type_at(m.from()) };
            for (i, &ptr) in conthist.iter().enumerate() {
                let scaled = bonus >> i;
                conthist_update(ptr, piece, m.to(), -scaled);
            }
        }
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
        self.pv_table[ply as usize][0] = mov;
        let next_len = (self.stack[ply + 1].pv_length.min(MAX_PLY - 1 - ply)) as usize;
        let (current, rest) = self.pv_table[ply as usize..].split_at_mut(1);
        for (dst, &src) in current[0][1..1 + next_len]
            .iter_mut()
            .zip(&rest[0][..next_len])
        {
            *dst = src;
        }
        self.stack[ply].pv_length = (next_len + 1) as u16;
    }

    #[inline(always)]
    fn get_lmr(&self, is_pv: bool, depth: u8, mov_index: u8) -> u8 {
        let (non_pv, pv) = self.lmr_table[depth.min(63) as usize][mov_index.min(63) as usize];
        if is_pv { pv } else { non_pv }
    }

    #[inline]
    fn eval_position(&mut self) -> i16 {
        let ply = self.ply();
        unsafe {
            std::hint::assert_unchecked(ply < MAX_PLY);
        }
        if self.stack[ply].acc_computed {
            return self.nnue_accumulator[ply as usize].eval(&self.board);
        }

        // Walk back to find nearest computed ancestor.
        let mut ancestor = ply;
        while ancestor > 0 {
            ancestor -= 1;
            if self.stack[ancestor].acc_computed {
                break;
            }
        }
        debug_assert!(
            self.stack[ancestor].acc_computed,
            "root ply 0 is always computed"
        );

        // Replay any intermediate plies if ancestor is further than 1 ply (rare)
        for intermediate_ply in (ancestor + 1)..ply {
            let (parent_acc, current_acc) = self
                .nnue_accumulator
                .split_at_mut(intermediate_ply as usize);
            let parent_acc = &parent_acc[intermediate_ply as usize - 1];
            let stack = &mut self.stack[intermediate_ply];
            current_acc[0].compute_from(parent_acc, stack.stack_move);
            stack.acc_computed = true;
        }

        // Fused compute + eval on the target ply
        let (parent_acc, current_acc) = self.nnue_accumulator.split_at_mut(ply as usize);
        let parent_acc = &parent_acc[ply as usize - 1];
        let stack = &mut self.stack[ply];
        let score = current_acc[0].compute_and_eval(parent_acc, stack.stack_move, &self.board);
        stack.acc_computed = true;
        score
    }

    fn nega_max<const IS_PV: bool>(
        &mut self,
        move_buffer: MoveListPtr,
        mut alpha: i16,
        beta: i16,
        depth: u8,
        can_null: bool,
    ) -> i16 {
        let ply = self.ply();
        if ply >= MAX_PLY - 1 {
            return self.eval_position();
        }

        let in_check = self.board.in_check();
        let has_non_pawn = self.board.has_non_pawn_material(self.board.to_play);
        self.stack[ply].pv_length = 0;
        self.stack.clear_killers(ply + 1);
        self.nodes_searched += 1;
        self.check_limits();

        if self.stopped || (ply > 0 && self.board.is_draw()) {
            return 0;
        }

        if depth == 0 {
            return self.qsearch(move_buffer, alpha, beta);
        }

        let (tt_move, mut static_eval) = match self.tt.probe(self.board.hash, ply) {
            Some(entry) => {
                if ply > 0
                    && let Some(score) = entry.cutoff(depth, alpha, beta)
                {
                    if !IS_PV {
                        return score;
                    }
                    if entry.flag() == TTFlag::Exact {
                        if !entry.mov.is_none() && self.board.legal(entry.mov) {
                            self.pv_table[ply as usize][0] = entry.mov;
                            self.stack[ply].pv_length = 1;
                        }
                        return score;
                    }
                }
                (entry.mov, entry.eval)
            }
            None => (Move::NONE, EVAL_NONE),
        };

        if static_eval == EVAL_NONE && !in_check {
            static_eval = self.eval_position();
        }

        self.stack[ply].eval = static_eval;

        let improving = if in_check || ply < 2 {
            false
        } else if self.stack.relative(ply, -2).eval != EVAL_NONE {
            static_eval > self.stack.relative(ply, -2).eval
        } else if self.stack.relative(ply, -4).eval != EVAL_NONE {
            static_eval > self.stack.relative(ply, -4).eval
        } else {
            true
        };

        // Reverse Futility Pruning
        let rfp_margin = (RFP_MARGIN_SLOPE * depth as i16)
            - (RFP_IMPROVING_BONUS * improving as i16)
            + if tt_move.is_none() {
                RFP_NO_TT_MARGIN
            } else {
                0
            };

        if !IS_PV
            && !in_check
            && depth <= RFP_DEPTH
            && beta < MATE_THRESHOLD
            && (tt_move.is_none() || !tt_move.is_tactical())
            && has_non_pawn
            && static_eval >= beta.saturating_add(rfp_margin)
        {
            return ((static_eval as i32 + beta as i32) / 2) as i16;
        }

        // Null Move Pruning
        if !IS_PV
            && !in_check
            && can_null
            && depth >= NMP_MIN_REDUCTION
            && static_eval >= beta
            && beta < MATE_THRESHOLD
            && has_non_pawn
        {
            let undo = self.board.make_null_move();
            self.stack[ply + 1].set_null_move();

            let eval_margin = (static_eval - beta).max(0);
            let eval_bonus =
                ((eval_margin / NMP_EVAL_DIVISOR).min(NMP_MAX_EVAL_BONUS as i16)) as u8;
            let reduction = NMP_MIN_REDUCTION + depth / NMP_DEPTH_DIVISOR + eval_bonus;

            let score = -self.nega_max::<false>(
                move_buffer,
                -beta,
                -beta + 1,
                depth.saturating_sub(reduction),
                false,
            );
            self.board.undo_null_move(undo);

            if self.stopped {
                return 0;
            }

            if score >= beta {
                return beta;
            }
        }

        // TODO: Tune LMP further once we gave better move ordering ~8 ELO
        let lmp_threshold = lmp_threshold(depth, improving);
        let futility_margin_eval =
            static_eval.saturating_add(FUTILITY_MARGIN.saturating_mul(depth as i16));
        let orig_alpha = alpha;
        let mut moves = MoveGenerator::new(move_buffer, tt_move);
        let mut legal_moves = 0;
        let mut quiet_moves = 0;
        let mut quiets_tried = QuietsTried::default();
        let mut skip_quiets = false;

        let mut best_score = -INFINITY;
        let mut best_move = Move::NONE;
        let killer_moves = self.stack.get_killers(ply);
        let conthist: ContHistPtrs = [
            self.stack[ply].conthist,
            self.stack.relative(ply, -1).conthist,
        ];

        while let Some(scored_mov) = moves.next(&self.board, killer_moves, &self.history, &conthist)
        {
            if self.stopped {
                return 0;
            }
            let mov = scored_mov.mov;
            let move_buffer = moves.next_ptr();
            if !self.board.legal(mov) {
                continue;
            }
            legal_moves += 1;
            if !mov.is_tactical() {
                quiet_moves += 1;
            }

            // SEE Pruning
            if !IS_PV
                && depth <= CAPTURE_SEE_MAX_DEPTH
                && mov.is_capture()
                && scored_mov.is_bad_capture()
                && !see_ge(mov, &self.board, depth as i32 * SEE_CAPTURE_MARGIN)
            {
                continue;
            }

            let move_gives_check = self.board.gives_check(mov);

            if skip_quiets && !mov.is_tactical() && !move_gives_check {
                continue;
            }

            // Move Count Based Pruning (Late Move Pruning)
            if !IS_PV
                && !in_check
                && depth <= LMP_MAX_DEPTH
                && quiet_moves > lmp_threshold
                && !mov.is_tactical()
                && !move_gives_check
                && best_score > -MATE_THRESHOLD
                && has_non_pawn
            {
                skip_quiets = true;
                continue;
            }

            // Futility Pruning
            if !IS_PV
                && depth < FUTILITY_MAX_DEPTH
                && legal_moves > FUTILITY_MIN_LEGAL_MOVES
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
                skip_quiets = true;
                continue;
            }

            // History Pruning
            if !IS_PV
                && depth <= HISTORY_PRUNING_DEPTH
                && !mov.is_tactical()
                && !move_gives_check
                && best_score > -MATE_THRESHOLD
                && scored_mov.score < -(depth as i16 * HISTORY_PRUNING_MARGIN)
            {
                skip_quiets = true;
                continue;
            }

            // Quiet Move SEE Pruning
            if !IS_PV
                && depth <= QUIET_SEE_MAX_DEPTH
                && !mov.is_tactical()
                && !move_gives_check
                && mov != killer_moves[0]
                && mov != killer_moves[1]
                && !see_ge(
                    mov,
                    &self.board,
                    QUIET_SEE_COEFF * (depth as i32) * (depth as i32),
                )
            {
                continue;
            }

            let moved_piece = self.board.piece_at(mov.from());
            let us = self.board.to_play;
            let undo = self.board.make_move_fast(mov, move_gives_check);
            let child_ply = ply + 1;
            self.stack[child_ply].set_move(mov, moved_piece, undo.captured_piece);
            self.stack[child_ply].conthist =
                moved_piece.map(|cp| self.cont_history.entry_ptr(cp.piece(), mov.to()));

            // --- Search the Move ---
            let mut score = -INFINITY;
            let mut do_full_search = true;

            // PVS Zero-Window Search with LMR
            if legal_moves > 1 {
                do_full_search = false;

                let mut reduction = 0;
                if legal_moves > LMR_MIN_LEGAL_MOVES
                    && depth >= LMR_MIN_DEPTH
                    && (mov.is_quiet() || scored_mov.is_bad_capture())
                    && !in_check
                    && !move_gives_check
                {
                    reduction = self.get_lmr(IS_PV, depth, legal_moves as u8) as i8;
                    reduction -= improving as i8;
                    // TODO: Test late-capture LMR (extend condition with is_late_capture)
                    // TODO: Test killer reduction (reduction -= is_killer as i8)
                    let hist = self.history.get(us, mov.from(), mov.to()) as i32
                        + conthist_score(&conthist, moved_piece.unwrap().piece(), mov.to()) as i32;
                    reduction -= (hist / LMR_HISTORY_DIVISOR) as i8;
                    // Avoid Ord::clamp here: it has an internal assert!(min <= max) that fails to inline
                    reduction = reduction.max(0).min(depth as i8 - 2);
                }

                let lmr_depth = depth - 1 - reduction as u8;

                score = -self.nega_max::<false>(move_buffer, -alpha - 1, -alpha, lmr_depth, true);

                if reduction > 0 && score > alpha {
                    score =
                        -self.nega_max::<false>(move_buffer, -alpha - 1, -alpha, depth - 1, true);
                }

                // ONLY for PV nodes, if it still beats alpha after full-depth search, open the window
                if IS_PV && score > alpha && score < beta {
                    do_full_search = true
                }
            }

            if do_full_search {
                score = -self.nega_max::<IS_PV>(move_buffer, -beta, -alpha, depth - 1, true);
            }
            self.board.undo_move(mov, undo);

            if self.stopped {
                return 0;
            }

            if score > best_score {
                best_score = score;
                if score > alpha {
                    alpha = score;
                    best_move = mov;
                    if IS_PV {
                        self.update_pv(ply, mov);
                    }
                }
            }

            if score >= beta {
                if mov.is_quiet() {
                    self.stack.set_killer(ply, mov);
                    self.update_quiet_history(mov, quiets_tried.as_slice(), &conthist, depth);
                }

                self.store_tt(mov, best_score, static_eval, depth, TTFlag::LowerBound);
                return best_score;
            }

            if mov.is_quiet() {
                quiets_tried.push_move(mov);
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

    fn qsearch(&mut self, move_buffer: MoveListPtr, mut alpha: i16, beta: i16) -> i16 {
        self.nodes_searched += 1;
        self.check_limits();
        if self.stopped {
            return 0;
        }

        let ply = self.ply();
        if ply >= MAX_PLY - 1 {
            return self.eval_position();
        }
        if ply > self.selective_depth as u16 {
            self.selective_depth = ply as u8;
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
                static_eval = self.eval_position();
            }
            if static_eval >= beta {
                self.store_tt(Move::NONE, static_eval, static_eval, 0, TTFlag::LowerBound);
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

        let mut moves = MoveGenerator::quiescence(move_buffer, tt_move);
        let mut best_move = Move::NONE;
        let killer_moves = self.stack.get_killers(ply);

        let no_conthist: ContHistPtrs = [None; CONTHIST_LAYERS];
        while let Some(scored_mov) =
            moves.next(&self.board, killer_moves, &self.history, &no_conthist)
        {
            let mov = scored_mov.mov;
            if !in_check && !mov.is_tactical() {
                continue;
            }
            if !in_check {
                // Delta Pruning
                let victim_val = if mov.flags() == MoveFlags::EnPassant {
                    piece_value(Piece::Pawn)
                } else {
                    self.board
                        .piece_at(mov.to())
                        .map_or(0, |p| piece_value(p.piece()))
                };

                let promo_val = if mov.is_promotion() {
                    piece_value(Piece::Queen) - piece_value(Piece::Pawn)
                } else {
                    0
                };

                if static_eval.saturating_add(victim_val + promo_val + DELTA_MARGIN) < alpha {
                    continue;
                }

                // SEE Pruning
                if tt_move != mov
                    && scored_mov.is_bad_capture()
                    && !see_ge(mov, &self.board, SEE_QSEARCH_MARGIN)
                {
                    continue;
                }
            }
            if !self.board.legal(mov) {
                continue;
            }

            let moved_piece = self.board.piece_at(mov.from());
            let undo = self.board.make_move(mov);
            let child_ply = ply + 1;
            self.stack[child_ply].set_move(mov, moved_piece, undo.captured_piece);

            let score = -self.qsearch(moves.next_ptr(), -beta, -alpha);
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
        }

        // In check with no legal moves is checkmate; best_score is still -INFINITY here.
        best_score
    }

    fn uci_info(&mut self, depth: u8, score: i16, start_time: Instant) -> String {
        let pv_str = self.pv_table[0][..self.stack[0].pv_length as usize]
            .iter()
            .map(|&m| crate::uci::format_move(m))
            .collect::<Vec<_>>()
            .join(" ");

        let elapsed = start_time.elapsed().as_millis().max(1);
        let nps = (self.nodes_searched as u128 * 1000 / elapsed) as u64;
        format!(
            "info depth {depth} seldepth {} score {} time {elapsed} nps {nps} nodes {} hashfull {} pv {pv_str}",
            self.selective_depth,
            crate::uci::format_score(score),
            self.nodes_searched,
            self.tt.hashfull(),
        )
    }

    fn resolve_best_move(&self, best: Move) -> Move {
        if best != Move::NONE && self.board.legal(best) {
            return best;
        }

        let pv_move = self.pv_table[0][0];
        if pv_move != Move::NONE && self.board.legal(pv_move) {
            return pv_move;
        }

        let tt_move = self
            .tt
            .probe(self.board.hash, 0)
            .map_or(Move::NONE, |e| e.mov);
        if tt_move != Move::NONE && self.board.legal(tt_move) {
            return tt_move;
        }

        gen_all_moves(&self.board)
            .as_slice()
            .iter()
            .copied()
            .map(|scored| scored.mov)
            .find(|&m| self.board.legal(m))
            .unwrap_or(Move::NONE)
    }
}
