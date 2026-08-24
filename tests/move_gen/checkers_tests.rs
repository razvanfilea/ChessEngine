use chess_base::prelude::*;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{Black, White, compute_checkers};

#[test]
fn test_no_checkers() {
    let board = Board::from_fen("8/8/8/8/8/8/8/K7 w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, 0);
}

#[test]
fn test_pawn_checker() {
    // White king on e4, black pawn on d5
    let board = Board::from_fen("8/8/8/3p4/4K3/8/8/8 w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, Sq::D5.bitboard());

    // Black king on e5, white pawn on f4
    let board = Board::from_fen("8/8/8/4k3/5P2/8/8/8 b - - 0 1").unwrap();
    let checkers = compute_checkers::<Black>(&board);
    assert_eq!(checkers, Sq::F4.bitboard());
}

#[test]
fn test_knight_checker() {
    // White king on e4, black knight on f6
    let board = Board::from_fen("8/8/5n2/8/4K3/8/8/8 w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, Sq::F6.bitboard());
}

#[test]
fn test_bishop_checker() {
    let board = Board::from_fen("b7/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, Sq::A8.bitboard());

    let board = Board::from_fen("b7/1P6/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, 0);
}

#[test]
fn test_rook_checker() {
    // White king on e4, black rook on e8
    let board = Board::from_fen("4r3/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, Sq::E8.bitboard());

    // Blocked
    let board = Board::from_fen("4r3/8/4P3/8/4K3/8/8/8 w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, 0);
}

#[test]
fn test_queen_checker() {
    // White king on e4, black queen on h1
    let board = Board::from_fen("8/8/8/8/4K3/8/8/7q w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, Sq::H1.bitboard());

    // White king on e4, black queen on a4
    let board = Board::from_fen("8/8/8/8/q3K3/8/8/8 w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, Sq::A4.bitboard());
}

#[test]
fn test_double_check() {
    // White king on e4, black rook on e8, black bishop on a8
    let board = Board::from_fen("b3r3/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, Sq::A8.bitboard() | Sq::E8.bitboard());
}

#[test]
fn test_own_pieces_dont_check() {
    // White king on e4, white rook on e8
    let board = Board::from_fen("4R3/8/8/8/4K3/8/8/8 w - - 0 1").unwrap();
    let checkers = compute_checkers::<White>(&board);
    assert_eq!(checkers, 0);
}
