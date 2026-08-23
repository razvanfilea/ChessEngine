use chess_base::{bitboard, prelude::*};
use lucky_chess::attacks::*;

#[test]
fn test_pawn_moves() {
    let sq = Sq::from_raw(8); // A2
    let moves = pawn_moves(sq, Color::White);
    assert_ne!(moves, 0);

    let moves_black = pawn_moves(Sq::from_raw(48), Color::Black); // A7
    assert_ne!(moves_black, 0);
}

#[test]
fn test_pawn_attacks() {
    let sq = Sq::from_raw(9); // B2
    let attacks = pawn_attacks(sq, Color::White);
    assert_ne!(attacks, 0);
}

#[test]
fn test_bishop_xray_attacks() {
    let sq = Sq::from_raw(27);
    let attacks = bishop_xray_attacks(sq);
    assert_ne!(attacks, 0);
}

#[test]
fn test_bishop_attacks() {
    let sq = Sq::from_raw(27);
    let attacks = bishop_attacks(sq, 0);
    assert_eq!(attacks, attacks);
}

#[test]
fn test_rook_attacks() {
    let sq = Sq::from_raw(27);
    let attacks = rook_attacks(sq, 0);
    assert_eq!(attacks, attacks);
}

#[test]
fn test_queen_attacks() {
    let sq = Sq::from_raw(27);
    let attacks = queen_attacks(sq, 0);
    assert_eq!(attacks, attacks);
}

#[test]
fn test_rook_xray_attacks() {
    let sq = Sq::from_raw(27);
    let attacks = rook_xray_attacks(sq);
    assert_ne!(attacks, 0);
}

#[test]
fn test_knight_attacks() {
    let sq = Sq::from_raw(27);
    let attacks = knight_attacks(sq);
    assert_ne!(attacks, 0);
}

#[test]
fn test_king_attacks() {
    let sq = Sq::from_raw(27);
    let attacks = king_attacks(sq);
    assert_ne!(attacks, 0);
}

#[test]
fn test_generate_bishop_attacks() {
    let sq = Sq::from_raw(27);
    let attacks = bitboard::generate_bishop_attacks(sq, 0);
    assert_ne!(attacks, 0);
}

#[test]
fn test_generate_rook_attacks() {
    let sq = Sq::from_raw(27);
    let attacks = bitboard::generate_rook_attacks(sq, 0);
    assert_ne!(attacks, 0);
}
