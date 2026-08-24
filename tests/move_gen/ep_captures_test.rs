use chess_base::prelude::MoveFlags;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{Captures, White, generate_moves};

#[test]
fn test_ep_captures_only() {
    // White pawn on e5, Black just played d7-d5.
    let board = Board::from_fen("4k3/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap();
    let moves = generate_moves::<White, Captures>(&board);
    let mut found_ep = false;
    for m in moves.as_slice() {
        if m.flags() == MoveFlags::EnPassant {
            found_ep = true;
        }
    }
    assert!(
        found_ep,
        "En passant capture should be generated when generating Captures"
    );
}
