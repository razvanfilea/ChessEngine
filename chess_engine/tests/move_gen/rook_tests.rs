use super::collect_piece_moves;
use chess_core::prelude::*;
use chess_engine::board::Board;
use chess_engine::move_gen::{Black, Captures, NonEvasions, Quiets, White, generate_rook_moves};

#[test]
fn test_rook_moves_center() {
    let board = Board::from_fen("8/8/8/8/3R4/8/8/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_rook_moves::<White, NonEvasions>,
    );
    assert_eq!(moves.len(), 14);
    assert!(moves.contains(&(Sq::D4, Sq::D5, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::D6, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::D7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::D8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::D3, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::D2, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::D1, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::E4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::F4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::G4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::H4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::C4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::B4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A4, MoveFlags::Quiet)));
}

#[test]
fn test_rook_moves_corner() {
    let board = Board::from_fen("r7/8/8/8/8/8/8/8 b - - 0 1").unwrap();
    let moves = collect_piece_moves::<Black, NonEvasions>(
        &board,
        generate_rook_moves::<Black, NonEvasions>,
    );
    assert_eq!(moves.len(), 14);
    assert!(moves.contains(&(Sq::A8, Sq::B8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::H8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::A7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::A1, MoveFlags::Quiet)));
}

#[test]
fn test_rook_blocked_by_friendly_pieces() {
    let board = Board::from_fen("8/8/8/3P4/2PRP3/3P4/8/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_rook_moves::<White, NonEvasions>,
    );
    let rook_moves: Vec<_> = moves.iter().filter(|m| m.0 == Sq::D4).collect();
    assert_eq!(rook_moves.len(), 0);
}

#[test]
fn test_rook_captures_enemy_pieces() {
    let board = Board::from_fen("8/8/3p4/8/1p1R2p1/8/3p4/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_rook_moves::<White, NonEvasions>,
    );
    let captures: Vec<_> = moves.iter().filter(|m| m.2 == MoveFlags::Capture).collect();
    assert_eq!(captures.len(), 4);
    assert!(captures.iter().any(|m| m.1 == Sq::D6));
    assert!(captures.iter().any(|m| m.1 == Sq::D2));
    assert!(captures.iter().any(|m| m.1 == Sq::B4));
    assert!(captures.iter().any(|m| m.1 == Sq::G4));
    assert!(!moves.iter().any(|m| m.1 == Sq::D7));
    assert!(!moves.iter().any(|m| m.1 == Sq::D1));
    assert!(!moves.iter().any(|m| m.1 == Sq::A4));
    assert!(!moves.iter().any(|m| m.1 == Sq::H4));
}

#[test]
fn test_rook_captures_only() {
    let board = Board::from_fen("8/8/3p4/8/1p1R2p1/8/3p4/8 w - - 0 1").unwrap();
    let moves =
        collect_piece_moves::<White, Captures>(&board, generate_rook_moves::<White, Captures>);
    assert_eq!(moves.len(), 4);
    for m in moves {
        assert_eq!(m.2, MoveFlags::Capture);
    }
}

#[test]
fn test_rook_quiets_only() {
    let board = Board::from_fen("8/8/3p4/8/1p1R2p1/8/3p4/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, Quiets>(&board, generate_rook_moves::<White, Quiets>);
    assert_eq!(moves.len(), 5);
    for m in moves {
        assert_eq!(m.2, MoveFlags::Quiet);
    }
}
