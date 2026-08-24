use chess_base::prelude::*;
use lucky_chess::board::Board;
use lucky_chess::move_gen::{
    generate_queen_moves, Black, Captures, MoveGenType, MoveList, NonEvasions, Player, Quiets,
    White,
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
    let new_pos = generate_queen_moves::<Us, Type>(board, target_mask, moves.as_ptr());
    moves.update_size(new_pos);
    let slice = moves.as_slice();
    let mut res = Vec::new();
    for m in slice {
        res.push((m.from(), m.to(), m.flags()));
    }
    res
}

#[test]
fn test_queen_moves_center() {
    let board = Board::from_fen("8/8/8/8/3Q4/8/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    // d4 queen on empty board has 27 moves (14 orthogonal + 13 diagonal)
    assert_eq!(moves.len(), 27);
    // Orthogonal
    assert!(moves.contains(&(Sq::D4, Sq::D8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::D1, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A4, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::H4, MoveFlags::Quiet)));
    // Diagonal
    assert!(moves.contains(&(Sq::D4, Sq::H8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A7, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::A1, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::D4, Sq::G1, MoveFlags::Quiet)));
}

#[test]
fn test_queen_moves_corner() {
    let board = Board::from_fen("q7/8/8/8/8/8/8/8 b - - 0 1").unwrap();
    let moves = get_moves::<Black, NonEvasions>(&board);
    // a8 queen on empty board: 7 rank 8 + 7 file A + 7 diagonal (b7..h1) = 21
    assert_eq!(moves.len(), 21);
    assert!(moves.contains(&(Sq::A8, Sq::H8, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::A1, MoveFlags::Quiet)));
    assert!(moves.contains(&(Sq::A8, Sq::H1, MoveFlags::Quiet)));
}

#[test]
fn test_queen_blocked_by_friendly_pieces() {
    // Queen on d4 surrounded by friendly pawns on all 8 neighboring squares
    let board = Board::from_fen("8/8/8/2PPP3/2PQP3/2PPP3/8/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    assert_eq!(moves.len(), 0);
}

#[test]
fn test_queen_captures_enemy_pieces() {
    // White Queen on d4, enemy pieces on d6 (north), f6 (NE), f4 (east), f2 (SE), d2 (south), b2 (SW), b4 (west), b6 (NW)
    let board = Board::from_fen("8/8/1p1p1p2/8/1p1Q1p2/8/1p1p1p2/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, NonEvasions>(&board);
    // Captures on all 8 ray directions (d6, f6, f4, f2, d2, b2, b4, b6) = 8 captures
    // Quiets on adjacent 8 squares (d5, e5, e4, e3, d3, c3, c4, c5) = 8 quiets
    let captures: Vec<_> = moves.iter().filter(|m| m.2 == MoveFlags::Capture).collect();
    let quiets: Vec<_> = moves.iter().filter(|m| m.2 == MoveFlags::Quiet).collect();
    assert_eq!(captures.len(), 8);
    assert_eq!(quiets.len(), 8);
    assert_eq!(moves.len(), 16);
}

#[test]
fn test_queen_captures_only() {
    let board = Board::from_fen("8/8/1p1p1p2/8/1p1Q1p2/8/1p1p1p2/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, Captures>(&board);
    assert_eq!(moves.len(), 8);
    for m in moves {
        assert_eq!(m.2, MoveFlags::Capture);
    }
}

#[test]
fn test_queen_quiets_only() {
    let board = Board::from_fen("8/8/1p1p1p2/8/1p1Q1p2/8/1p1p1p2/8 w - - 0 1").unwrap();
    let moves = get_moves::<White, Quiets>(&board);
    assert_eq!(moves.len(), 8);
    for m in moves {
        assert_eq!(m.2, MoveFlags::Quiet);
    }
}
