use chess_base::Move;
use std::time::Instant;

use crate::eval::INFINITY;
use crate::{board::Board, eval::eval_board, move_gen::MoveGenerator};

pub fn search(mut board: Board, max_depth: i16) -> Move {
    let start_time = Instant::now();
    let mut overall_best_move = None;
    let mut overall_best_score = -INFINITY;

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
            let score = -nega_max(&mut board, -beta, -alpha, current_depth - 1);
            board.undo_move(pv_move, undo);

            best_move = Some(pv_move);
            best_score = score;
            alpha = score;
        }

        while let Some(mov) = moves.next(&mut board) {
            if Some(mov) == overall_best_move || !board.legal(mov) {
                continue;
            }

            let undo = board.make_move(mov);
            let score = -nega_max(&mut board, -beta, -alpha, current_depth - 1);
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

        if let Some(m) = overall_best_move {
            println!(
                "info depth {current_depth} score {} time {} pv {}",
                crate::uci::format_score(overall_best_score),
                start_time.elapsed().as_millis(),
                crate::uci::format_move(m)
            );
        }
    }

    overall_best_move.unwrap()
}

fn nega_max(board: &mut Board, mut alpha: i16, beta: i16, depth: i16) -> i16 {
    if board.ply > 0 && board.is_draw() {
        return 0;
    }

    if depth == 0 {
        return quiesce(board, alpha, beta);
    }

    let mut moves = MoveGenerator::default();
    let mut legal_moves = 0;

    let mut best_score = -INFINITY;

    while let Some(mov) = moves.next(board) {
        if !board.legal(mov) {
            continue;
        }
        legal_moves += 1;

        let undo = board.make_move(mov);
        let score = -nega_max(board, -beta, -alpha, depth - 1);
        if score > best_score {
            best_score = score;
            if score > alpha {
                alpha = score;
            }
        }
        board.undo_move(mov, undo);

        if score >= beta {
            return best_score;
        }
    }

    if legal_moves == 0 {
        if board.checkers != 0 {
            return -INFINITY + depth;
        } else {
            return 0; // Stalemate
        }
    }

    best_score
}

fn quiesce(board: &mut Board, mut alpha: i16, beta: i16) -> i16 {
    let in_check = board.checkers != 0;
    if !in_check {
        let static_eval = eval_board(board);

        if static_eval >= beta {
            return static_eval;
        }
        if static_eval > alpha {
            alpha = static_eval;
        }
    }

    let mut moves = MoveGenerator::quiescence();
    let mut legal_moves = 0;

    while let Some(mov) = moves.next(board) {
        if !in_check && (!mov.is_capture() && !mov.is_promotion()) {
            continue;
        }
        if !board.legal(mov) {
            continue;
        }
        legal_moves += 1;

        let undo = board.make_move(mov);
        let score = -quiesce(board, -beta, -alpha);
        board.undo_move(mov, undo);

        if score >= beta {
            return score;
        }
        if score > alpha {
            alpha = score;
        }
    }

    if in_check && legal_moves == 0 {
        return -INFINITY;
    }

    alpha
}
