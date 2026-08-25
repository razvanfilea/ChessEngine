use chess_base::prelude::MoveFlags;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{Evasions, White, gen_moves};

#[test]
fn test_ep_evasion() {
    let board = Board::from_fen("4k3/8/8/3pP3/2K5/8/8/8 w - d6 0 1").unwrap();
    let moves = gen_moves::<White, Evasions>(&board);
    let mut found_ep = false;
    for m in moves.as_slice() {
        if m.flags() == MoveFlags::EnPassant {
            found_ep = true;
        }
    }
    assert!(
        found_ep,
        "En passant capture should be generated as an evasion"
    );
}
