use super::collect_piece_moves;
use chess_core::prelude::*;
use chess_engine::board::Board;
use chess_engine::move_gen::{Black, Captures, NonEvasions, Quiets, White, generate_bishop_moves};

#[test]
fn test_bishop_moves_center() {
    let board = Board::from_fen("8/8/8/8/3B4/8/8/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_bishop_moves::<White, NonEvasions>,
    );
    assert_eq!(moves.len(), 13);
    assert!(moves.contains(&(Sq::D4, Sq::E5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::F6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::G7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::H8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::C5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::B6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::E3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::F2, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::G1, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::C3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::B2, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A1, MoveFlags::Quiet)));
}

#[test]
fn test_bishop_moves_corner() {
    let board = Board::from_fen("8/8/8/8/8/8/8/b7 b - - 0 1").unwrap();
    let moves = collect_piece_moves::<Black, NonEvasions>(
        &board,
        generate_bishop_moves::<Black, NonEvasions>,
    );
    assert_eq!(moves.len(), 7);
    assert!(moves.contains(&(Sq::A1, Sq::B2, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::C3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::D4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::E5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::F6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::G7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A1, Sq::H8, MoveFlags::Quiet)));
}

#[test]
fn test_bishop_blocked_by_friendly_pieces() {
    let board = Board::from_fen("8/8/8/2P1P3/3B4/2P1P3/8/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_bishop_moves::<White, NonEvasions>,
    );
    assert_eq!(moves.len(), 0);
}

#[test]
fn test_bishop_captures_enemy_pieces() {
    let board = Board::from_fen("8/8/8/2p1p3/3B4/2p1p3/8/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_bishop_moves::<White, NonEvasions>,
    );
    assert_eq!(moves.len(), 4);
    assert!(moves.contains(&(Sq::D4, Sq::C5, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::E5, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::C3, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::E3, MoveFlags::Capture)));
}

#[test]
fn test_bishop_captures_only() {
    let board = Board::from_fen("8/8/5p2/8/3B4/8/8/8 w - - 0 1").unwrap();
    let moves =
        collect_piece_moves::<White, Captures>(&board, generate_bishop_moves::<White, Captures>);
    assert_eq!(moves.len(), 1);
    assert!(moves.contains(&(Sq::D4, Sq::F6, MoveFlags::Capture)));
}

#[test]
fn test_bishop_quiets_only() {
    let board = Board::from_fen("8/8/5p2/8/3B4/8/8/8 w - - 0 1").unwrap();
    let moves =
        collect_piece_moves::<White, Quiets>(&board, generate_bishop_moves::<White, Quiets>);
    assert_eq!(moves.len(), 10);
    assert!(!moves.contains(&(Sq::D4, Sq::F6, MoveFlags::Capture)));
    assert!(!moves.contains(&(Sq::D4, Sq::G7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::E5, MoveFlags::Quiet)));
}
