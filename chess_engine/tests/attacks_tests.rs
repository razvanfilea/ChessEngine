use chess_core::{bitboard, prelude::*};
use chess_engine::attacks::*;

#[test]
fn test_pawn_attacks() {
    let e4 = Sq::E4;
    let white_attacks = pawn_attacks(e4, Color::White);
    let expected_white = Sq::D5.bitboard() | Sq::F5.bitboard();
    assert_eq!(white_attacks, expected_white);

    let black_attacks = pawn_attacks(e4, Color::Black);
    let expected_black = Sq::D3.bitboard() | Sq::F3.bitboard();
    assert_eq!(black_attacks, expected_black);

    let a4 = Sq::A4;
    let white_a4 = pawn_attacks(a4, Color::White);
    assert_eq!(white_a4, Sq::B5.bitboard());

    let h4 = Sq::H4;
    let black_h4 = pawn_attacks(h4, Color::Black);
    assert_eq!(black_h4, Sq::G3.bitboard());

    assert_eq!(pawn_attacks(Sq::A8, Color::White), 0);
    assert_eq!(pawn_attacks(Sq::H1, Color::Black), 0);
}

#[test]
fn test_knight_attacks() {
    let e4_attacks = knight_attacks(Sq::E4);
    let expected = Sq::D6.bitboard()
        | Sq::F6.bitboard()
        | Sq::C5.bitboard()
        | Sq::G5.bitboard()
        | Sq::C3.bitboard()
        | Sq::G3.bitboard()
        | Sq::D2.bitboard()
        | Sq::F2.bitboard();
    assert_eq!(e4_attacks, expected);

    let a1_attacks = knight_attacks(Sq::A1);
    let expected_a1 = Sq::B3.bitboard() | Sq::C2.bitboard();
    assert_eq!(a1_attacks, expected_a1);

    let h8_attacks = knight_attacks(Sq::H8);
    let expected_h8 = Sq::G6.bitboard() | Sq::F7.bitboard();
    assert_eq!(h8_attacks, expected_h8);
}

#[test]
fn test_king_attacks() {
    let e4_attacks = king_attacks(Sq::E4);
    let expected = Sq::D5.bitboard()
        | Sq::E5.bitboard()
        | Sq::F5.bitboard()
        | Sq::D4.bitboard()
        | Sq::F4.bitboard()
        | Sq::D3.bitboard()
        | Sq::E3.bitboard()
        | Sq::F3.bitboard();
    assert_eq!(e4_attacks, expected);

    let h8_attacks = king_attacks(Sq::H8);
    let expected_h8 = Sq::G8.bitboard() | Sq::G7.bitboard() | Sq::H7.bitboard();
    assert_eq!(h8_attacks, expected_h8);

    let a4_attacks = king_attacks(Sq::A4);
    let expected_a4 = Sq::A5.bitboard()
        | Sq::B5.bitboard()
        | Sq::B4.bitboard()
        | Sq::A3.bitboard()
        | Sq::B3.bitboard();
    assert_eq!(a4_attacks, expected_a4);
}

#[test]
fn test_bishop_xray_attacks() {
    let e4_attacks = bishop_xray_attacks(Sq::E4);
    let expected = bitboard::bb_from_dir(Dir::NorthWest, Sq::E4)
        | bitboard::bb_from_dir(Dir::NorthEast, Sq::E4)
        | bitboard::bb_from_dir(Dir::SouthWest, Sq::E4)
        | bitboard::bb_from_dir(Dir::SouthEast, Sq::E4);
    assert_eq!(e4_attacks, expected);

    let a1_attacks = bishop_xray_attacks(Sq::A1);
    let expected_a1 = bitboard::bb_from_dir(Dir::NorthEast, Sq::A1);
    assert_eq!(a1_attacks, expected_a1);
}

#[test]
fn test_rook_xray_attacks() {
    let e4_attacks = rook_xray_attacks(Sq::E4);
    let expected = bitboard::bb_from_dir(Dir::North, Sq::E4)
        | bitboard::bb_from_dir(Dir::South, Sq::E4)
        | bitboard::bb_from_dir(Dir::East, Sq::E4)
        | bitboard::bb_from_dir(Dir::West, Sq::E4);
    assert_eq!(e4_attacks, expected);

    let a1_attacks = rook_xray_attacks(Sq::A1);
    let expected_a1 =
        bitboard::bb_from_dir(Dir::North, Sq::A1) | bitboard::bb_from_dir(Dir::East, Sq::A1);
    assert_eq!(a1_attacks, expected_a1);
}

#[test]
fn test_slider_attacks_vs_reference() {
    let occupancies: [u64; 4] = [
        0,
        0x00FF_00FF_00FF_00FF,
        0xFF00_FF00_FF00_FF00,
        0x8142_2418_1824_4281,
    ];

    for sq_idx in 0..64u8 {
        let sq = unsafe { Sq::from_raw_unchecked(sq_idx) };
        for &occ in &occupancies {
            assert_eq!(
                bishop_attacks(sq, occ),
                bitboard::bb_bishop_attacks(sq, occ),
                "bishop mismatch at {sq:?} occ={occ:#x}"
            );
            assert_eq!(
                rook_attacks(sq, occ),
                bitboard::bb_rook_attacks(sq, occ),
                "rook mismatch at {sq:?} occ={occ:#x}"
            );
            assert_eq!(
                queen_attacks(sq, occ),
                bitboard::bb_bishop_attacks(sq, occ) | bitboard::bb_rook_attacks(sq, occ),
                "queen mismatch at {sq:?} occ={occ:#x}"
            );
        }
    }
}
