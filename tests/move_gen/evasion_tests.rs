use chess_base::prelude::*;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{generate_moves, White, Evasions, MoveGenType, MoveList};

#[test]
fn test_double_check_evasions() {
    // White king on e4, checked by black rook on e8 and black bishop on a8
    let board = Board::from_fen("b3r3/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let moves = generate_moves::<White, Evasions>(&board);
    // King can move to d3, d4, d5, e5, f5, f4, f3, e3. Some of them might be attacked?
    // Wait, generate_king_moves generates all pseudo-legal moves for the king.
    // So the move list should not be empty.
    assert_ne!(moves.as_slice().len(), 0);
}
