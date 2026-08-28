use chess_core::prelude::*;
use chess_engine::board::Board;

#[test]
fn test_no_attackers() {
    let board = Board::from_fen("8/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
    let checkers = board.generate_attackers(Sq::A1, Color::Black, board.occupied());
    assert_eq!(checkers, 0);
}

#[test]
fn test_pawn_attackers() {
    // White king on e4, black pawn on d5
    let board = Board::from_fen("8/8/8/3p4/4K3/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, Sq::D5.bitboard());

    // Black king on e5, white pawn on f4
    let board = Board::from_fen("8/8/8/4k3/5P2/8/8/8 b - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E5, Color::White, board.occupied());
    assert_eq!(attackers, Sq::F4.bitboard());

    // Multiple pawn attackers on d4
    let board = Board::from_fen("8/8/8/2p1p3/3P4/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::D4, Color::Black, board.occupied());
    assert_eq!(attackers, Sq::C5.bitboard() | Sq::E5.bitboard());
}

#[test]
fn test_knight_attackers() {
    // White king on e4, black knight on f6
    let board = Board::from_fen("8/8/5n2/8/4K3/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, Sq::F6.bitboard());

    // Multiple knights attacking d5
    let board = Board::from_fen("8/2n1n3/8/3K4/8/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::D5, Color::Black, board.occupied());
    assert_eq!(attackers, Sq::C7.bitboard() | Sq::E7.bitboard());
}

#[test]
fn test_bishop_attackers() {
    let board = Board::from_fen("b7/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, Sq::A8.bitboard());

    let board = Board::from_fen("b7/1P6/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, 0);
}

#[test]
fn test_rook_attackers() {
    // White king on e4, black rook on e8
    let board = Board::from_fen("4r3/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, Sq::E8.bitboard());

    // Blocked
    let board = Board::from_fen("4r3/8/4P3/8/4K3/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, 0);
}

#[test]
fn test_queen_attackers() {
    // White king on e4, black queen on h1
    let board = Board::from_fen("8/8/8/8/4K3/8/8/7q w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, Sq::H1.bitboard());

    // White king on e4, black queen on a4
    let board = Board::from_fen("8/8/8/8/q3K3/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, Sq::A4.bitboard());
}

#[test]
fn test_king_attackers() {
    // King on e4, enemy king on e5
    let board = Board::from_fen("8/8/8/4k3/4K3/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, Sq::E5.bitboard());
}

#[test]
fn test_double_check() {
    // White king on e4, black rook on e8, black bishop on a8
    let board = Board::from_fen("b3r3/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, Sq::A8.bitboard() | Sq::E8.bitboard());
}

#[test]
fn test_own_pieces_dont_attack() {
    // White king on e4, white rook on e8
    let board = Board::from_fen("4R3/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let attackers = board.generate_attackers(Sq::E4, Color::Black, board.occupied());
    assert_eq!(attackers, 0);
}

#[test]
fn test_custom_occupancy() {
    // White king on e4, black rook on e8, blocker on e6
    let board = Board::from_fen("4r3/8/4P3/8/4K3/8/8/8 w - - 0 1").unwrap();
    // With board occupancy, it's blocked
    assert_eq!(
        board.generate_attackers(Sq::E4, Color::Black, board.occupied()),
        0
    );
    // If e6 blocker is removed from occupied bitboard, e8 attacks e4
    let occ_without_e6 = board.occupied() ^ Sq::E6.bitboard();
    assert_eq!(
        board.generate_attackers(Sq::E4, Color::Black, occ_without_e6),
        Sq::E8.bitboard()
    );
}
