use super::collect_piece_moves;
use chess_base::prelude::*;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{Black, Captures, NonEvasions, Quiets, White, generate_knight_moves};

#[test]
fn test_knight_moves_center() {
    let board = Board::from_fen("8/8/8/8/3N4/8/8/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_knight_moves::<White, NonEvasions>,
    );
    assert_eq!(moves.len(), 8);
    assert!(moves.contains(&(Sq::D4, Sq::C6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::E6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::B5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::F5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::B3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::F3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::C2, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::E2, MoveFlags::Quiet)));
}

#[test]
fn test_knight_moves_corner() {
    let board = Board::from_fen("n7/8/8/8/8/8/8/8 b - - 0 1").unwrap();
    let moves = collect_piece_moves::<Black, NonEvasions>(
        &board,
        generate_knight_moves::<Black, NonEvasions>,
    );
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&(Sq::A8, Sq::B6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::C7, MoveFlags::Quiet)));
}

#[test]
fn test_knight_captures() {
    let board = Board::from_fen("8/8/2p1p3/1p3p2/3N4/1p3p2/2p1p3/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_knight_moves::<White, NonEvasions>,
    );
    assert_eq!(moves.len(), 8);
    assert!(moves.contains(&(Sq::D4, Sq::C6, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::E6, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::B5, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::F5, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::B3, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::F3, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::C2, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::D4, Sq::E2, MoveFlags::Capture)));
}

#[test]
fn test_knight_blocked_by_own_pieces() {
    let board = Board::from_fen("8/8/2P1P3/1P3P2/3N4/1P3P2/2P1P3/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_knight_moves::<White, NonEvasions>,
    );
    assert_eq!(moves.len(), 0);
}

#[test]
fn test_gen_captures_only() {
    let board = Board::from_fen("8/8/8/8/4n3/8/3P1P2/8 b - - 0 1").unwrap();
    let moves =
        collect_piece_moves::<Black, Captures>(&board, generate_knight_moves::<Black, Captures>);
    assert_eq!(moves.len(), 2);
    assert!(moves.contains(&(Sq::E4, Sq::D2, MoveFlags::Capture)));
    assert!(moves.contains(&(Sq::E4, Sq::F2, MoveFlags::Capture)));
}

#[test]
fn test_gen_quiets_only() {
    let board = Board::from_fen("8/8/8/8/4n3/8/3P1P2/8 b - - 0 1").unwrap();
    let moves =
        collect_piece_moves::<Black, Quiets>(&board, generate_knight_moves::<Black, Quiets>);
    assert_eq!(moves.len(), 6);
    assert!(moves.contains(&(Sq::E4, Sq::C5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::C3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::D6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::F6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::G5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::E4, Sq::G3, MoveFlags::Quiet)));
}
