use chess_base::prelude::*;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{
    Black, Captures, MoveGenType, MoveList, NonEvasions, Player, Quiets, White, generate_rook_moves,
};

fn get_moves<Us: Player, Type: MoveGenType>(board: &Board) -> Vec<(Sq, Sq, MoveFlags)> {
    let them = !Us::COLOR;
    let target_mask = if Type::EVASIONS {
        !0
    } else {
        let mut mask = 0;
        if Type::CAPTURES {
            mask |= board.colors(them);
        }
        if Type::QUIETS {
            mask |= !board.occupied();
        }
        mask
    };

    let mut moves = MoveList::default();
    let new_pos = generate_rook_moves::<Us, Type>(board, target_mask, moves.as_ptr());
    moves.update_size(new_pos);
    let slice = moves.as_slice();
    let mut res = Vec::new();
    for m in slice {
        res.push((m.from(), m.to(), m.flags()));
    }
    res
}

#[test]
fn test_rook_moves_center() {
    let board = Board::from_fen("8/8/8/8/3R4/8/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    // d4 rook on empty board has 14 orthogonal moves (7 rank, 7 file):
    // North: d5, d6, d7, d8 (4)
    // South: d3, d2, d1 (3)
    // East: e4, f4, g4, h4 (4)
    // West: c4, b4, a4 (3)
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
    let moves = get_moves::<Black, NonEvasions>(&board);
    // a8 rook on empty board: 7 rank 8 (b8..h8) + 7 file A (a7..a1) = 14
    assert_eq!(moves.len(), 14);
    assert!(moves.contains(&(Sq::A8, Sq::B8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::H8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::A7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::A1, MoveFlags::Quiet)));
}

#[test]
fn test_rook_blocked_by_friendly_pieces() {
    // Rook on d4 surrounded by friendly pawns on d5, d3, e4, c4
    let board = Board::from_fen("8/8/8/3P4/2PRP3/3P4/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    let rook_moves: Vec<_> = moves.iter().filter(|m| m.0 == Sq::D4).collect();
    assert_eq!(rook_moves.len(), 0);
}

#[test]
fn test_rook_captures_enemy_pieces() {
    // White Rook on d4, enemy pieces on d6, d2, b4, g4
    let board = Board::from_fen("8/8/3p4/8/1p1R2p1/8/3p4/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    // Captures: d6, d2, b4, g4 (4)
    // Quiets: d5 (1), d3 (1), c4 (1), e4 (1), f4 (1) = 5
    let captures: Vec<_> = moves.iter().filter(|m| m.2 == MoveFlags::Capture).collect();
    assert_eq!(captures.len(), 4);
    assert!(captures.iter().any(|m| m.1 == Sq::D6));
    assert!(captures.iter().any(|m| m.1 == Sq::D2));
    assert!(captures.iter().any(|m| m.1 == Sq::B4));
    assert!(captures.iter().any(|m| m.1 == Sq::G4));

    // Beyond captured pieces shouldn't be reachable
    assert!(!moves.iter().any(|m| m.1 == Sq::D7));
    assert!(!moves.iter().any(|m| m.1 == Sq::D1));
    assert!(!moves.iter().any(|m| m.1 == Sq::A4));
    assert!(!moves.iter().any(|m| m.1 == Sq::H4));
}

#[test]
fn test_rook_captures_only() {
    let board = Board::from_fen("8/8/3p4/8/1p1R2p1/8/3p4/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, Captures>(&board);
    assert_eq!(moves.len(), 4);
    for m in moves {
        assert_eq!(m.2, MoveFlags::Capture);
    }
}

#[test]
fn test_rook_quiets_only() {
    let board = Board::from_fen("8/8/3p4/8/1p1R2p1/8/3p4/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, Quiets>(&board);
    assert_eq!(moves.len(), 5);
    for m in moves {
        assert_eq!(m.2, MoveFlags::Quiet);
    }
}
