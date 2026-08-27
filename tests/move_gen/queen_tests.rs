use super::collect_piece_moves;
use chess_base::prelude::*;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{Black, Captures, NonEvasions, Quiets, White, generate_queen_moves};

#[test]
fn test_queen_moves_center() {
    let board = Board::from_fen("8/8/8/8/3Q4/8/8/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_queen_moves::<White, NonEvasions>,
    );
    assert_eq!(moves.len(), 27);
    assert!(moves.contains(&(Sq::D4, Sq::D8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::D1, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::H4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::H8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A1, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::G1, MoveFlags::Quiet)));
}

#[test]
fn test_queen_moves_corner() {
    let board = Board::from_fen("q7/8/8/8/8/8/8/8 b - - 0 1").unwrap();
    let moves = collect_piece_moves::<Black, NonEvasions>(
        &board,
        generate_queen_moves::<Black, NonEvasions>,
    );
    assert_eq!(moves.len(), 21);
    assert!(moves.contains(&(Sq::A8, Sq::H8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::A1, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::H1, MoveFlags::Quiet)));
}

#[test]
fn test_queen_blocked_by_friendly_pieces() {
    let board = Board::from_fen("8/8/8/2PPP3/2PQP3/2PPP3/8/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_queen_moves::<White, NonEvasions>,
    );
    assert_eq!(moves.len(), 0);
}

#[test]
fn test_queen_captures_enemy_pieces() {
    let board = Board::from_fen("8/8/1p1p1p2/8/1p1Q1p2/8/1p1p1p2/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, NonEvasions>(
        &board,
        generate_queen_moves::<White, NonEvasions>,
    );
    let captures: Vec<_> = moves.iter().filter(|m| m.2 == MoveFlags::Capture).collect();
    let quiets: Vec<_> = moves.iter().filter(|m| m.2 == MoveFlags::Quiet).collect();
    assert_eq!(captures.len(), 8);
    assert_eq!(quiets.len(), 8);
    assert_eq!(moves.len(), 16);
}

#[test]
fn test_queen_captures_only() {
    let board = Board::from_fen("8/8/1p1p1p2/8/1p1Q1p2/8/1p1p1p2/8 w - - 0 1").unwrap();
    let moves =
        collect_piece_moves::<White, Captures>(&board, generate_queen_moves::<White, Captures>);
    assert_eq!(moves.len(), 8);
    for m in moves {
        assert_eq!(m.2, MoveFlags::Capture);
    }
}

#[test]
fn test_queen_quiets_only() {
    let board = Board::from_fen("8/8/1p1p1p2/8/1p1Q1p2/8/1p1p1p2/8 w - - 0 1").unwrap();
    let moves = collect_piece_moves::<White, Quiets>(&board, generate_queen_moves::<White, Quiets>);
    assert_eq!(moves.len(), 8);
    for m in moves {
        assert_eq!(m.2, MoveFlags::Quiet);
    }
}
