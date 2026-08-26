use chess_base::Move;

use crate::{board::Board, eval::eval_board, move_gen::MoveGenerator};

const INFINITY: i16 = 30_000;

pub fn search(mut board: Board, depth: i16) -> Move {
    let mut moves = MoveGenerator::default();

    let mut best_move = None;
    let mut best_score = -INFINITY;

    let mut alpha = -INFINITY;

    while let Some(mov) = moves.next(&mut board) {
        if !board.legal(mov) {
            continue;
        }

        let undo = board.make_move(mov);
        let score = -nega_max(&mut board, -INFINITY, -alpha, depth - 1);
        if score > best_score {
            best_score = score;
            best_move = Some(mov);
            if score > alpha {
                alpha = score;
            }
        }
        board.undo_move(mov, undo);
    }

    return best_move.unwrap();
}

pub fn nega_max(board: &mut Board, mut alpha: i16, beta: i16, depth: i16) -> i16 {
    if depth == 0 {
        return eval_board(board);
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

    return best_score;
}
