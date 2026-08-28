use chess_core::prelude::*;
use chess_engine::board::Board;
use chess_engine::move_gen::{Black, Captures, Evasions, NonEvasions, White, gen_moves};

#[test]
fn test_castling_white_and_black_full_rights() {
    // Both White and Black have full rights and empty paths
    let white_board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();
    let white_moves = gen_moves::<White, NonEvasions>(&white_board);

    let has_w_kingside = white_moves
        .as_slice()
        .iter()
        .any(|m| m.flags() == MoveFlags::CastleKing && m.from() == Sq::E1 && m.to() == Sq::G1);
    let has_w_queenside = white_moves
        .as_slice()
        .iter()
        .any(|m| m.flags() == MoveFlags::CastleQueen && m.from() == Sq::E1 && m.to() == Sq::C1);

    assert!(
        has_w_kingside,
        "White Kingside castling should be generated"
    );
    assert!(
        has_w_queenside,
        "White Queenside castling should be generated"
    );

    let black_board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1").unwrap();
    let black_moves = gen_moves::<Black, NonEvasions>(&black_board);

    let has_b_kingside = black_moves
        .as_slice()
        .iter()
        .any(|m| m.flags() == MoveFlags::CastleKing && m.from() == Sq::E8 && m.to() == Sq::G8);
    let has_b_queenside = black_moves
        .as_slice()
        .iter()
        .any(|m| m.flags() == MoveFlags::CastleQueen && m.from() == Sq::E8 && m.to() == Sq::C8);

    assert!(
        has_b_kingside,
        "Black Kingside castling should be generated"
    );
    assert!(
        has_b_queenside,
        "Black Queenside castling should be generated"
    );
}

#[test]
fn test_castling_blocked_by_friendly_piece() {
    // White: bishop on f1 blocks kingside, knight on d1 blocks queenside
    let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R2NKB1R w KQkq - 0 1").unwrap();
    let moves = gen_moves::<White, NonEvasions>(&board);

    let has_kingside = moves
        .as_slice()
        .iter()
        .any(|m| m.flags() == MoveFlags::CastleKing);
    let has_queenside = moves
        .as_slice()
        .iter()
        .any(|m| m.flags() == MoveFlags::CastleQueen);

    assert!(
        !has_kingside,
        "Kingside castling blocked by f1 should not be generated"
    );
    assert!(
        !has_queenside,
        "Queenside castling blocked by d1 should not be generated"
    );
}

#[test]
fn test_castling_blocked_by_b1_square() {
    // White: knight on b1 blocks queenside path
    let board = Board::from_fen("r3k2r/8/8/8/8/8/8/RN2K2R w KQkq - 0 1").unwrap();
    let moves = gen_moves::<White, NonEvasions>(&board);

    let has_kingside = moves
        .as_slice()
        .iter()
        .any(|m| m.flags() == MoveFlags::CastleKing);
    let has_queenside = moves
        .as_slice()
        .iter()
        .any(|m| m.flags() == MoveFlags::CastleQueen);

    assert!(has_kingside, "Kingside should be open");
    assert!(
        !has_queenside,
        "Queenside blocked on b1 should not be generated"
    );
}

#[test]
fn test_castling_partial_rights() {
    // White only has K right (kingside)
    let k_only = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w K - 0 1").unwrap();
    let moves_k = gen_moves::<White, NonEvasions>(&k_only);
    assert!(
        moves_k
            .as_slice()
            .iter()
            .any(|m| m.flags() == MoveFlags::CastleKing)
    );
    assert!(
        !moves_k
            .as_slice()
            .iter()
            .any(|m| m.flags() == MoveFlags::CastleQueen)
    );

    // White only has Q right (queenside)
    let q_only = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w Q - 0 1").unwrap();
    let moves_q = gen_moves::<White, NonEvasions>(&q_only);
    assert!(
        !moves_q
            .as_slice()
            .iter()
            .any(|m| m.flags() == MoveFlags::CastleKing)
    );
    assert!(
        moves_q
            .as_slice()
            .iter()
            .any(|m| m.flags() == MoveFlags::CastleQueen)
    );

    // No rights
    let no_rights = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w - - 0 1").unwrap();
    let moves_none = gen_moves::<White, NonEvasions>(&no_rights);
    assert!(
        !moves_none
            .as_slice()
            .iter()
            .any(|m| m.flags() == MoveFlags::CastleKing)
    );
    assert!(
        !moves_none
            .as_slice()
            .iter()
            .any(|m| m.flags() == MoveFlags::CastleQueen)
    );
}

#[test]
fn test_castling_not_generated_in_captures_or_evasions() {
    let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();

    let cap_moves = gen_moves::<White, Captures>(&board);
    assert!(
        !cap_moves
            .as_slice()
            .iter()
            .any(|m| m.flags() == MoveFlags::CastleKing || m.flags() == MoveFlags::CastleQueen)
    );

    let eva_moves = gen_moves::<White, Evasions>(&board);
    assert!(
        !eva_moves
            .as_slice()
            .iter()
            .any(|m| m.flags() == MoveFlags::CastleKing || m.flags() == MoveFlags::CastleQueen)
    );
}
