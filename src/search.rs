use chess_base::Move;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::eval::INFINITY;
use crate::{board::Board, eval::eval_board, move_gen::MoveGenerator};

const MAX_PLY: u16 = 64;
const MAX_KILLER_MOVES: usize = 2;

pub type KillerMoves = [Move; MAX_KILLER_MOVES];
type KillerMovesArray = [KillerMoves; MAX_PLY as usize];
struct SearchInfo {
    nodes_searched: u64,
    killer_moves: KillerMovesArray,
    root_ply: u16,
}

impl SearchInfo {
    fn killer_index(&self, ply: u16) -> Option<usize> {
        let idx = (ply - self.root_ply) as usize;
        (idx < MAX_PLY as usize).then_some(idx)
    }

    fn get_killer_moves(&self, ply: u16) -> KillerMoves {
        match self.killer_index(ply) {
            Some(idx) => self.killer_moves[idx],
            None => [Move::default(); MAX_KILLER_MOVES],
        }
    }

    fn set_killer_move(&mut self, ply: u16, current_move: Move) {
        let Some(idx) = self.killer_index(ply) else {
            return;
        };
        let [first_killer, second_killer] = &mut self.killer_moves[idx];

        if *first_killer == current_move {
            return;
        }

        *second_killer = *first_killer;
        *first_killer = current_move;
    }
}

pub fn search(mut board: Board, max_depth: u16, stop_requested: Arc<AtomicBool>) -> Move {
    let start_time = Instant::now();
    let mut overall_best_move = None;
    let mut overall_best_score = -INFINITY;
    let mut info = SearchInfo {
        nodes_searched: 0,
        killer_moves: [[Move::default(); MAX_KILLER_MOVES]; MAX_PLY as usize],
        root_ply: board.ply,
    };

    for current_depth in 1..=max_depth {
        let mut best_move = None;
        let mut best_score = -INFINITY;
        let mut moves = MoveGenerator::default();

        let mut alpha = -INFINITY;
        let beta = INFINITY;

        if let Some(pv_move) = overall_best_move
            && board.legal(pv_move)
        {
            let undo = board.make_move(pv_move);
            let score = -nega_max(&mut board, &mut info, -beta, -alpha, current_depth - 1);
            board.undo_move(pv_move, undo);

            best_move = Some(pv_move);
            best_score = score;
            alpha = score;
        }

        while let Some(mov) = moves.next(&board, info.get_killer_moves(board.ply)) {
            if Some(mov) == overall_best_move || !board.legal(mov) {
                continue;
            }

            let undo = board.make_move(mov);
            let score = -nega_max(&mut board, &mut info, -beta, -alpha, current_depth - 1);
            if score > best_score {
                best_score = score;
                best_move = Some(mov);
                if score > alpha {
                    alpha = score;
                }
            }
            board.undo_move(mov, undo);
        }

        if let Some(mov) = best_move {
            overall_best_move = Some(mov);
            overall_best_score = best_score;
        }

        if let Some(mov) = overall_best_move {
            let time = start_time.elapsed().as_millis();
            let nps = (info.nodes_searched as u128 / time.max(1)) as u64;
            println!(
                "info depth {current_depth} score {} time {} nps {} nodes {} pv {}",
                crate::uci::format_score(overall_best_score),
                time,
                nps,
                info.nodes_searched,
                crate::uci::format_move(mov)
            );

            if stop_requested.load(Ordering::Relaxed) {
                return mov;
            }
        }
    }

    overall_best_move.unwrap()
}

fn nega_max(
    board: &mut Board,
    info: &mut SearchInfo,
    mut alpha: i16,
    beta: i16,
    depth: u16,
) -> i16 {
    if board.ply > 0 && board.is_draw() {
        return 0;
    }

    if depth == 0 {
        return quiesce(board, info, alpha, beta);
    }

    let mut moves = MoveGenerator::default();
    let mut legal_moves = 0;

    let mut best_score = -INFINITY;

    while let Some(mov) = moves.next(board, info.get_killer_moves(board.ply)) {
        if !board.legal(mov) {
            continue;
        }
        legal_moves += 1;

        let undo = board.make_move(mov);
        let score = -nega_max(board, info, -beta, -alpha, depth - 1);
        if score > best_score {
            best_score = score;
            if score > alpha {
                alpha = score;
            }
        }
        board.undo_move(mov, undo);

        if score >= beta {
            if !mov.is_capture() {
                info.set_killer_move(board.ply, mov);
            }
            info.nodes_searched += legal_moves;
            return best_score;
        }
    }

    info.nodes_searched += legal_moves;

    if legal_moves == 0 {
        if board.checkers != 0 {
            return -INFINITY + depth as i16;
        } else {
            return 0; // Stalemate
        }
    }

    best_score
}

fn quiesce(board: &mut Board, info: &mut SearchInfo, mut alpha: i16, beta: i16) -> i16 {
    let in_check = board.checkers != 0;

    // Stand-pat: not available while in check, since every evasion must be considered.
    let mut best_score = if in_check {
        -INFINITY
    } else {
        let static_eval = eval_board(board);
        if static_eval >= beta {
            return static_eval;
        }
        if static_eval > alpha {
            alpha = static_eval;
        }
        static_eval
    };

    let mut moves = MoveGenerator::quiescence();

    while let Some(mov) = moves.next(board, info.get_killer_moves(board.ply)) {
        if !in_check && (!mov.is_capture() && !mov.is_promotion()) {
            continue;
        }
        if !board.legal(mov) {
            continue;
        }
        info.nodes_searched += 1;

        let undo = board.make_move(mov);
        let score = -quiesce(board, info, -beta, -alpha);
        board.undo_move(mov, undo);

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
