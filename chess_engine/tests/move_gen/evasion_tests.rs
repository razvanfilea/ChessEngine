use chess_core::prelude::*;
use chess_engine::board::Board;
use chess_engine::move_gen::{Black, Evasions, White, gen_moves};

#[test]
fn test_double_check_only_king_moves() {
    // White king on e4 checked by black rook on e8 and black bishop on a8.
    // White also has a knight on d2 that could otherwise move.
    let board = Board::from_fen("b3r3/8/8/8/4K3/8/3N4/8 w - - 0 1").unwrap();
    let moves = gen_moves::<White, Evasions>(&board);

    assert_ne!(moves.as_slice().len(), 0);
    // In double check, ONLY king moves must be generated (from == E4)
    for m in moves.as_slice() {
        assert_eq!(
            m.from(),
            Sq::E4,
            "Only King moves are valid in double check"
        );
    }
}

#[test]
fn test_single_check_interposition_blocking() {
    // White King on e1, Black Rook on e8 gives check.
    // White Knight on b1 can jump to d2 or c3 (c3 doesn't block, d2 doesn't block, but if knight is on c3 it can jump to e2 or e4 to block).
    // Let's place White Knight on c3: attacks e4, e2, d5, b5, a4, a2, b1, d1.
    // The ray between e1 and e8 is {e2, e3, e4, e5, e6, e7}.
    // Knight on c3 can jump to e2 and e4 to block!
    let board = Board::from_fen("4r3/8/8/8/8/2N5/8/4K3 w - - 0 1").unwrap();
    let moves = gen_moves::<White, Evasions>(&board);

    // Knight moves should only be e2 and e4
    let knight_moves: Vec<_> = moves
        .as_slice()
        .iter()
        .filter(|m| m.from() == Sq::C3)
        .collect();
    assert_eq!(knight_moves.len(), 2);
    assert!(knight_moves.iter().any(|m| m.to() == Sq::E2));
    assert!(knight_moves.iter().any(|m| m.to() == Sq::E4));

    // King moves should also be present (e.g. d1, d2, f1, f2)
    let king_moves: Vec<_> = moves
        .as_slice()
        .iter()
        .filter(|m| m.from() == Sq::E1)
        .collect();
    assert!(!king_moves.is_empty());
}

#[test]
fn test_single_check_capture_of_checker() {
    // White King on e1, Black Knight checks from d3.
    // White pawn on c2 can capture on d3 (north_east).
    // White pawn on e2 can capture on d3 (north_west).
    let board = Board::from_fen("8/8/8/8/8/3n4/2P1P3/4K3 w - - 0 1").unwrap();
    let moves = gen_moves::<White, Evasions>(&board);

    let pawn_c2_cap = moves
        .as_slice()
        .iter()
        .any(|m| m.from() == Sq::C2 && m.to() == Sq::D3 && m.is_capture());
    let pawn_e2_cap = moves
        .as_slice()
        .iter()
        .any(|m| m.from() == Sq::E2 && m.to() == Sq::D3 && m.is_capture());

    assert!(
        pawn_c2_cap,
        "Pawn on c2 should be able to capture checker on d3"
    );
    assert!(
        pawn_e2_cap,
        "Pawn on e2 should be able to capture checker on d3"
    );
}

#[test]
fn test_black_evasions_single_check() {
    // Black King on e8, White Rook on e1 gives check.
    // Black Knight on c6 can block on e7 or e5.
    let board = Board::from_fen("4k3/8/2n5/8/8/8/8/4R3 b - - 0 1").unwrap();
    let moves = gen_moves::<Black, Evasions>(&board);

    let knight_blocks: Vec<_> = moves
        .as_slice()
        .iter()
        .filter(|m| m.from() == Sq::C6)
        .collect();
    assert_eq!(knight_blocks.len(), 2);
    assert!(knight_blocks.iter().any(|m| m.to() == Sq::E7));
    assert!(knight_blocks.iter().any(|m| m.to() == Sq::E5));

    let king_moves: Vec<_> = moves
        .as_slice()
        .iter()
        .filter(|m| m.from() == Sq::E8)
        .collect();
    assert!(!king_moves.is_empty());
}
