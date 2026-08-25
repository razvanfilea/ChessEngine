use chess_base::prelude::*;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{Black, Captures, NonEvasions, Quiets, White, gen_moves};

#[test]
fn test_king_center_moves() {
    // White king on e4 on an empty board
    let board = Board::from_fen("8/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let moves = gen_moves::<White, NonEvasions>(&board);

    // King on e4 can move to 8 adjacent squares: d3, d4, d5, e5, f5, f4, f3, e3
    assert_eq!(moves.as_slice().len(), 8);
    for m in moves.as_slice() {
        assert_eq!(m.from(), Sq::E4);
        assert_eq!(m.flags(), MoveFlags::Quiet);
    }
}

#[test]
fn test_king_corner_moves() {
    // White king on a1 on an empty board
    let board = Board::from_fen("8/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
    let moves = gen_moves::<White, NonEvasions>(&board);

    // King on a1 can move to 3 adjacent squares: a2, b1, b2
    assert_eq!(moves.as_slice().len(), 3);
    for m in moves.as_slice() {
        assert_eq!(m.from(), Sq::A1);
        assert_eq!(m.flags(), MoveFlags::Quiet);
    }
}

#[test]
fn test_king_blocked_by_friendly_pieces() {
    // White king on e1 surrounded by friendly pieces on d1, d2, e2, f2, f1
    // (No castling rights in FEN)
    let board = Board::from_fen("8/8/8/8/8/8/3PPP2/3RKR2 w - - 0 1").unwrap();
    let moves = gen_moves::<White, NonEvasions>(&board);

    // King moves should be 0 because all adjacent squares are occupied by friendly pieces
    let king_moves: Vec<_> = moves
        .as_slice()
        .iter()
        .filter(|m| m.from() == Sq::E1)
        .collect();
    assert_eq!(king_moves.len(), 0);
}

#[test]
fn test_king_captures_and_flags() {
    // White king on e4, enemy black pawn on e5 and black knight on d4
    let board = Board::from_fen("8/8/8/4p3/3nK3/8/8/8 w - - 0 1").unwrap();
    let moves = gen_moves::<White, NonEvasions>(&board);

    let king_moves = moves.as_slice();
    assert_eq!(king_moves.len(), 8);

    let captures: Vec<_> = king_moves.iter().filter(|m| m.is_capture()).collect();
    assert_eq!(captures.len(), 2);
    assert!(captures.iter().any(|m| m.to() == Sq::E5));
    assert!(captures.iter().any(|m| m.to() == Sq::D4));

    let quiets: Vec<_> = king_moves
        .iter()
        .filter(|m| m.flags() == MoveFlags::Quiet)
        .collect();
    assert_eq!(quiets.len(), 6);
}

#[test]
fn test_king_captures_only_gen_type() {
    // White king on e4 with enemy piece on e5
    let board = Board::from_fen("8/8/8/4p3/4K3/8/8/8 w - - 0 1").unwrap();
    let moves = gen_moves::<White, Captures>(&board);

    // Only the capture on e5 should be generated
    assert_eq!(moves.as_slice().len(), 1);
    assert_eq!(moves.as_slice()[0].from(), Sq::E4);
    assert_eq!(moves.as_slice()[0].to(), Sq::E5);
    assert_eq!(moves.as_slice()[0].flags(), MoveFlags::Capture);
}

#[test]
fn test_king_quiets_only_gen_type() {
    // White king on e4 with enemy piece on e5
    let board = Board::from_fen("8/8/8/4p3/4K3/8/8/8 w - - 0 1").unwrap();
    let moves = gen_moves::<White, Quiets>(&board);

    // 7 quiet moves (excluding e5)
    assert_eq!(moves.as_slice().len(), 7);
    for m in moves.as_slice() {
        assert_ne!(m.to(), Sq::E5);
        assert_eq!(m.flags(), MoveFlags::Quiet);
    }
}

#[test]
fn test_black_king_moves() {
    // Black king on e8 with enemy piece on d8
    let board = Board::from_fen("3Rk3/8/8/8/8/8/8/8 b - - 0 1").unwrap();
    let moves = gen_moves::<Black, NonEvasions>(&board);

    // King on e8 has adjacent squares: d8 (capture), d7, e7, f7, f8 (quiets)
    let captures: Vec<_> = moves.as_slice().iter().filter(|m| m.is_capture()).collect();
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0].from(), Sq::E8);
    assert_eq!(captures[0].to(), Sq::D8);
}
