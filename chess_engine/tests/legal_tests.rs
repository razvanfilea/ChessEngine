use chess_core::prelude::*;
use chess_engine::board::Board;

#[test]
fn test_unpinned_piece_moves() {
    let board = Board::start_pos();
    // e2e4 is quiet double push
    let m = Move::new(Sq::E2, Sq::E4, MoveFlags::DoublePawn);
    assert!(board.legal(m), "e2e4 from startpos is legal");

    // g1f3 is quiet knight move
    let m = Move::new(Sq::G1, Sq::F3, MoveFlags::Quiet);
    assert!(board.legal(m), "g1f3 from startpos is legal");
}

#[test]
fn test_absolute_pin_rook_movement() {
    // White King on e1, White Rook on e3, Black Rook on e8
    let board = Board::from_fen("4r3/8/8/8/8/4R3/8/4K3 w - - 0 1").unwrap();

    // Rook moving along the pin ray (e.g. e3e4, e3e2, or capturing e3xe8)
    let move_along_ray_up = Move::new(Sq::E3, Sq::E4, MoveFlags::Quiet);
    assert!(
        board.legal(move_along_ray_up),
        "Pinned rook moving along pin ray up should be legal"
    );

    let move_along_ray_down = Move::new(Sq::E3, Sq::E2, MoveFlags::Quiet);
    assert!(
        board.legal(move_along_ray_down),
        "Pinned rook moving along pin ray down should be legal"
    );

    let capture_pinner = Move::new(Sq::E3, Sq::E8, MoveFlags::Capture);
    assert!(
        board.legal(capture_pinner),
        "Pinned rook capturing pinner on e8 should be legal"
    );

    // Rook moving off the pin ray (e.g. e3d3, e3f3, e3a3)
    let move_off_ray_left = Move::new(Sq::E3, Sq::D3, MoveFlags::Quiet);
    assert!(
        !board.legal(move_off_ray_left),
        "Pinned rook moving left off pin ray should be illegal"
    );

    let move_off_ray_right = Move::new(Sq::E3, Sq::F3, MoveFlags::Quiet);
    assert!(
        !board.legal(move_off_ray_right),
        "Pinned rook moving right off pin ray should be illegal"
    );
}

#[test]
fn test_absolute_pin_bishop_movement() {
    // White King on e1, White Bishop on d2, Black Bishop on a5 (diagonal a5-e1)
    let board = Board::from_fen("8/8/8/b7/8/8/3B4/4K3 w - - 0 1").unwrap();

    // Bishop moving along diagonal ray to c3
    let move_c3 = Move::new(Sq::D2, Sq::C3, MoveFlags::Quiet);
    assert!(
        board.legal(move_c3),
        "Pinned bishop moving along ray to c3 should be legal"
    );

    // Bishop capturing pinner on a5
    let capture_a5 = Move::new(Sq::D2, Sq::A5, MoveFlags::Capture);
    assert!(
        board.legal(capture_a5),
        "Pinned bishop capturing pinner on a5 should be legal"
    );

    // Bishop moving off the ray (e.g. d2e3, d2f4)
    let move_off_ray = Move::new(Sq::D2, Sq::E3, MoveFlags::Quiet);
    assert!(
        !board.legal(move_off_ray),
        "Pinned bishop moving off pin ray should be illegal"
    );
}

#[test]
fn test_absolute_pin_knight_cannot_move() {
    // White King on e1, White Knight on e2, Black Rook on e8
    let board = Board::from_fen("4r3/8/8/8/8/8/4N3/4K3 w - - 0 1").unwrap();

    // Knight jumps off file (e2d4, e2f4, e2c3, e2g3)
    let knight_d4 = Move::new(Sq::E2, Sq::D4, MoveFlags::Quiet);
    assert!(
        !board.legal(knight_d4),
        "Pinned knight moving to d4 should be illegal"
    );

    let knight_c3 = Move::new(Sq::E2, Sq::C3, MoveFlags::Quiet);
    assert!(
        !board.legal(knight_c3),
        "Pinned knight moving to c3 should be illegal"
    );
}

#[test]
fn test_absolute_pin_pawn_movement() {
    // White King on e1, White Pawn on e2, Black Rook on e8
    let board = Board::from_fen("4r3/8/8/8/8/8/4P3/4K3 w - - 0 1").unwrap();

    // Pawn push along the file
    let push_e3 = Move::new(Sq::E2, Sq::E3, MoveFlags::Quiet);
    assert!(
        board.legal(push_e3),
        "Pinned pawn pushing forward along pin ray should be legal"
    );

    let push_e4 = Move::new(Sq::E2, Sq::E4, MoveFlags::DoublePawn);
    assert!(
        board.legal(push_e4),
        "Pinned pawn double pushing along pin ray should be legal"
    );

    // Diagonal pawn pin: White King on e1, White Pawn on d2, Black Bishop on c3
    let board_diag = Board::from_fen("8/8/8/8/8/2b5/3P4/4K3 w - - 0 1").unwrap();

    // Pawn captures on c3 along the pin ray (capturing the pinner)
    let cap_c3 = Move::new(Sq::D2, Sq::C3, MoveFlags::Capture);
    assert!(
        board_diag.legal(cap_c3),
        "Pawn capturing pinner along diagonal pin ray should be legal"
    );

    // Pawn pushing d2d3 leaves diagonal pin ray
    let push_d3 = Move::new(Sq::D2, Sq::D3, MoveFlags::Quiet);
    assert!(
        !board_diag.legal(push_d3),
        "Pawn pushing forward off diagonal pin ray should be illegal"
    );
}

#[test]
fn test_king_moving_to_safe_square() {
    let board = Board::from_fen("8/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();
    let move_d1 = Move::new(Sq::E1, Sq::D1, MoveFlags::Quiet);
    let move_f1 = Move::new(Sq::E1, Sq::F1, MoveFlags::Quiet);
    let move_e2 = Move::new(Sq::E1, Sq::E2, MoveFlags::Quiet);

    assert!(board.legal(move_d1), "King moving to d1 should be legal");
    assert!(board.legal(move_f1), "King moving to f1 should be legal");
    assert!(board.legal(move_e2), "King moving to e2 should be legal");
}

#[test]
fn test_king_moving_into_check() {
    // White King on e1, Black Rook on d8 controlling d-file
    let board = Board::from_fen("3r4/8/8/8/8/8/8/4K3 w - - 0 1").unwrap();

    let move_into_check_d1 = Move::new(Sq::E1, Sq::D1, MoveFlags::Quiet);
    assert!(
        !board.legal(move_into_check_d1),
        "King moving to d1 (attacked by rook on d8) should be illegal"
    );

    let move_into_check_d2 = Move::new(Sq::E1, Sq::D2, MoveFlags::Quiet);
    assert!(
        !board.legal(move_into_check_d2),
        "King moving to d2 (attacked by rook on d8) should be illegal"
    );

    let move_safe_f1 = Move::new(Sq::E1, Sq::F1, MoveFlags::Quiet);
    assert!(
        board.legal(move_safe_f1),
        "King moving to f1 (safe square) should be legal"
    );
}

#[test]
fn test_king_capturing_defended_and_undefended_piece() {
    // White King on e1, Black Pawn on e2 defended by Black Pawn on d3
    let board = Board::from_fen("8/8/8/8/8/3p4/4p3/4K3 w - - 0 1").unwrap();

    let capture_defended = Move::new(Sq::E1, Sq::E2, MoveFlags::Capture);
    assert!(
        !board.legal(capture_defended),
        "King capturing defended pawn on e2 should be illegal"
    );

    // White King on e1, undefended Black Pawn on e2
    let board_undefended = Board::from_fen("8/8/8/8/8/8/4p3/4K3 w - - 0 1").unwrap();
    let capture_undefended = Move::new(Sq::E1, Sq::E2, MoveFlags::Capture);
    assert!(
        board_undefended.legal(capture_undefended),
        "King capturing undefended pawn on e2 should be legal"
    );
}

#[test]
fn test_castling_legality() {
    // Position with all castling rights open and safe
    let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1").unwrap();

    let castle_ks = Move::new(Sq::E1, Sq::G1, MoveFlags::CastleKing);
    assert!(
        board.legal(castle_ks),
        "Kingside castling when path is safe should be legal"
    );

    let castle_qs = Move::new(Sq::E1, Sq::C1, MoveFlags::CastleQueen);
    assert!(
        board.legal(castle_qs),
        "Queenside castling when path is safe should be legal"
    );

    // In check on e1: Black Rook on e8
    let board_in_check = Board::from_fen("4r3/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    assert!(
        !board_in_check.legal(castle_ks),
        "Castling while in check should be illegal"
    );
    assert!(
        !board_in_check.legal(castle_qs),
        "Castling while in check should be illegal"
    );

    // Passing through check: Black Rook on f8 attacks f1
    let board_through_check = Board::from_fen("5r2/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    assert!(
        !board_through_check.legal(castle_ks),
        "Kingside castling through check on f1 should be illegal"
    );

    // Landing in check: Black Rook on g8 attacks g1
    let board_into_check = Board::from_fen("6r1/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    assert!(
        !board_into_check.legal(castle_ks),
        "Kingside castling landing in check on g1 should be illegal"
    );

    // Queenside: b1 attacked (b1 is not on king path [e1, d1, c1]) -> legal
    let board_b1_attacked = Board::from_fen("1r6/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    assert!(
        board_b1_attacked.legal(castle_qs),
        "Queenside castling with b1 attacked should be legal"
    );

    // Queenside: d1 attacked (king passes through d1) -> illegal
    let board_d1_attacked = Board::from_fen("3r4/8/8/8/8/8/8/R3K2R w KQ - 0 1").unwrap();
    assert!(
        !board_d1_attacked.legal(castle_qs),
        "Queenside castling through check on d1 should be illegal"
    );
}

#[test]
fn test_en_passant_horizontal_pin_illegal() {
    // White King on e5, White Pawn on f5, Black Pawn on g5 (moved g7-g5), Black Rook on a5
    // FEN: Black just played g7-g5, en passant square is g6
    let board = Board::from_fen("8/8/8/r3K1pP/8/8/8/8 w - g6 0 1").unwrap();

    let ep_move = Move::new(Sq::H5, Sq::G6, MoveFlags::EnPassant);
    assert!(
        !board.legal(ep_move),
        "En passant exposing horizontal rook check on rank 5 should be illegal"
    );
}

#[test]
fn test_en_passant_legal() {
    // White King on e1, White Pawn on e5, Black Pawn on d5 (ep square d6), no pins
    let board = Board::from_fen("8/8/8/3pP3/8/8/8/4K3 w - d6 0 1").unwrap();

    let ep_move = Move::new(Sq::E5, Sq::D6, MoveFlags::EnPassant);
    assert!(
        board.legal(ep_move),
        "Normal en passant capture should be legal"
    );
}

#[test]
fn test_black_pins_and_king_moves() {
    // Black King on e8, Black Rook on e6, White Rook on e1 (Black to play)
    let board = Board::from_fen("4k3/8/4r3/8/8/8/8/4R3 b - - 0 1").unwrap();

    let rook_along_ray = Move::new(Sq::E6, Sq::E4, MoveFlags::Quiet);
    assert!(
        board.legal(rook_along_ray),
        "Black pinned rook moving along pin ray should be legal"
    );

    let rook_off_ray = Move::new(Sq::E6, Sq::D6, MoveFlags::Quiet);
    assert!(
        !board.legal(rook_off_ray),
        "Black pinned rook moving off pin ray should be illegal"
    );

    // White rook on e1 pins Black rook on e6.
    // If Black king moves to d8 or f8 (safe), it's legal.
    let king_safe = Move::new(Sq::E8, Sq::D8, MoveFlags::Quiet);
    assert!(
        board.legal(king_safe),
        "Black king moving to d8 (safe square) should be legal"
    );

    // Black King on e8, White Rook on d1 controlling d-file (Black to play)
    let board_d_file = Board::from_fen("4k3/8/8/8/8/8/8/3R4 b - - 0 1").unwrap();
    let king_into_check = Move::new(Sq::E8, Sq::D8, MoveFlags::Quiet);
    assert!(
        !board_d_file.legal(king_into_check),
        "Black king stepping onto d8 (attacked by d1 rook) should be illegal"
    );
}

#[test]
fn test_black_castling_legality() {
    // Both sides full rights, safe
    let board = Board::from_fen("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1").unwrap();

    let castle_ks = Move::new(Sq::E8, Sq::G8, MoveFlags::CastleKing);
    assert!(
        board.legal(castle_ks),
        "Black Kingside castling when safe should be legal"
    );

    let castle_qs = Move::new(Sq::E8, Sq::C8, MoveFlags::CastleQueen);
    assert!(
        board.legal(castle_qs),
        "Black Queenside castling when safe should be legal"
    );

    // In check: White Rook on e1
    let board_in_check = Board::from_fen("r3k2r/8/8/8/8/8/8/4R3 b kq - 0 1").unwrap();
    assert!(
        !board_in_check.legal(castle_ks),
        "Black castling in check should be illegal"
    );

    // Through check: White Rook on f1 attacks f8
    let board_through = Board::from_fen("r3k2r/8/8/8/8/8/8/5R2 b kq - 0 1").unwrap();
    assert!(
        !board_through.legal(castle_ks),
        "Black castling through check on f8 should be illegal"
    );

    // Into check: White Rook on g1 attacks g8
    let board_into = Board::from_fen("r3k2r/8/8/8/8/8/8/6R1 b kq - 0 1").unwrap();
    assert!(
        !board_into.legal(castle_ks),
        "Black castling into check on g8 should be illegal"
    );
}

#[test]
fn test_queen_pin_movement() {
    // White King on e1, White Bishop on e2, Black Queen on e8
    let board = Board::from_fen("4q3/8/8/8/8/8/4B3/4K3 w - - 0 1").unwrap();

    // Bishop on e2 is pinned vertically to King on e1 by Queen on e8.
    // Bishop cannot move diagonally (e2d3, e2f3)
    let move_diag = Move::new(Sq::E2, Sq::D3, MoveFlags::Quiet);
    assert!(
        !board.legal(move_diag),
        "Bishop pinned vertically by queen cannot move diagonally"
    );

    // White King on e1, White Rook on d2, Black Queen on a5 (diagonal pin a5-e1)
    let board_diag = Board::from_fen("8/8/8/q7/8/8/3R4/4K3 w - - 0 1").unwrap();

    // Rook on d2 is pinned diagonally by Queen on a5.
    // Rook cannot move along rank or file (e.g. d2d4 or d2e2)
    let rook_file_move = Move::new(Sq::D2, Sq::D4, MoveFlags::Quiet);
    assert!(
        !board_diag.legal(rook_file_move),
        "Rook pinned diagonally by queen cannot move along file"
    );
}
