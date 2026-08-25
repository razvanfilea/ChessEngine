use chess_base::prelude::*;

use crate::{
    board::Board,
    move_gen::{Black, Evasions, MoveList, NonEvasions, White, generate_moves},
};

pub fn perft(board: &Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let us = board.to_play;
    let king_sq = board.king_sq(us);
    let in_check = board.generate_attackers(king_sq, !us, board.occupied()) != 0;

    let mut moves = MoveList::default();
    let ptr = match (us, in_check) {
        (Color::White, true) => generate_moves::<White, Evasions>(board, moves.as_ptr()),
        (Color::White, false) => generate_moves::<White, NonEvasions>(board, moves.as_ptr()),
        (Color::Black, true) => generate_moves::<Black, Evasions>(board, moves.as_ptr()),
        (Color::Black, false) => generate_moves::<Black, NonEvasions>(board, moves.as_ptr()),
    };
    moves.update_size(ptr);

    let mut nodes = 0;
    for mov in moves.as_slice() {
        if !board.legal(*mov) {
            continue;
        }

        if depth == 1 {
            nodes += 1;
            continue;
        }

        let mut new_board = board.clone();
        new_board.make_move(*mov);
        nodes += perft(&new_board, depth - 1);
    }

    nodes
}
