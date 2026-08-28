use chess_core::prelude::*;
use chess_engine::board::Board;
use chess_engine::move_gen::{MoveGenType, MoveList, MoveListPtr, Player};

pub fn collect_piece_moves<Us: Player, Type: MoveGenType>(
    board: &Board,
    generate: fn(&Board, u64, MoveListPtr) -> MoveListPtr,
) -> Vec<(Sq, Sq, MoveFlags)> {
    let them = !Us::COLOR;
    let target_mask = if Type::EVASIONS {
        !0
    } else {
        let mut mask = 0;
        if Type::CAPTURES {
            mask |= board.colors(them);
        }
        if Type::QUIETS {
            mask |= !board.occupied();
        }
        mask
    };
    let mut moves = MoveList::default();
    let end = generate(board, target_mask, moves.as_ptr());
    moves.update_size(end);
    moves
        .as_slice()
        .iter()
        .map(|m| (m.from(), m.to(), m.flags()))
        .collect()
}

pub mod bishop_tests;
pub mod castling_tests;
pub mod checkers_tests;
pub mod ep_captures_test;
pub mod ep_evasion_test;
pub mod evasion_tests;
pub mod king_tests;
pub mod knight_tests;
pub mod move_list_tests;
pub mod pawn_tests;
pub mod queen_tests;
pub mod rook_tests;
