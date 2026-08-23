use lucky_chess::attacks::*;
use lucky_chess::square::Sq;
use lucky_chess::types::Color;

#[test]
fn test_pawn_moves() {
    let e4 = Sq::E4;
    // White pawn moves
    let white_moves = pawn_moves(e4, Color::White);
    assert_eq!(white_moves, Sq::E5.bitboard());

    // Black pawn moves
    let black_moves = pawn_moves(e4, Color::Black);
    assert_eq!(black_moves, Sq::E3.bitboard());

    // Top edge for white
    let a8 = Sq::A8;
    assert_eq!(pawn_moves(a8, Color::White), 0);

    // Bottom edge for black
    let a1 = Sq::A1;
    assert_eq!(pawn_moves(a1, Color::Black), 0);
}

#[test]
fn test_pawn_attacks() {
    // White pawn on E4 attacks D5 and F5
    let e4 = Sq::E4;
    let white_attacks = pawn_attacks(e4, Color::White);
    let expected_white = Sq::D5.bitboard() | Sq::F5.bitboard();
    assert_eq!(white_attacks, expected_white);

    // Black pawn on E4 attacks D3 and F3
    let black_attacks = pawn_attacks(e4, Color::Black);
    let expected_black = Sq::D3.bitboard() | Sq::F3.bitboard();
    assert_eq!(black_attacks, expected_black);

    // Edge cases: pawns on A file (can only attack right)
    let a4 = Sq::A4;
    let white_a4 = pawn_attacks(a4, Color::White);
    assert_eq!(white_a4, Sq::B5.bitboard());

    // Pawns on H file (can only attack left)
    let h4 = Sq::H4;
    let black_h4 = pawn_attacks(h4, Color::Black);
    assert_eq!(black_h4, Sq::G3.bitboard());
    
    // Edges of the board vertically
    assert_eq!(pawn_attacks(Sq::A8, Color::White), 0);
    assert_eq!(pawn_attacks(Sq::H1, Color::Black), 0);
}

#[test]
fn test_knight_attacks() {
    // Center of the board: E4
    // Knight at E4 can move to D6, F6, C5, G5, C3, G3, D2, F2
    let e4_attacks = knight_attacks(Sq::E4);
    let expected = Sq::D6.bitboard() | Sq::F6.bitboard()
                 | Sq::C5.bitboard() | Sq::G5.bitboard()
                 | Sq::C3.bitboard() | Sq::G3.bitboard()
                 | Sq::D2.bitboard() | Sq::F2.bitboard();
    assert_eq!(e4_attacks, expected);

    // Corner of the board: A1
    // Knight at A1 can move to B3, C2
    let a1_attacks = knight_attacks(Sq::A1);
    let expected_a1 = Sq::B3.bitboard() | Sq::C2.bitboard();
    assert_eq!(a1_attacks, expected_a1);
    
    // Another corner: H8
    let h8_attacks = knight_attacks(Sq::H8);
    let expected_h8 = Sq::G6.bitboard() | Sq::F7.bitboard();
    assert_eq!(h8_attacks, expected_h8);
}

#[test]
fn test_king_attacks() {
    // Center of the board: E4
    let e4_attacks = king_attacks(Sq::E4);
    let expected = Sq::D5.bitboard() | Sq::E5.bitboard() | Sq::F5.bitboard()
                 | Sq::D4.bitboard()                     | Sq::F4.bitboard()
                 | Sq::D3.bitboard() | Sq::E3.bitboard() | Sq::F3.bitboard();
    assert_eq!(e4_attacks, expected);

    // Corner of the board: H8
    let h8_attacks = king_attacks(Sq::H8);
    let expected_h8 = Sq::G8.bitboard() | Sq::G7.bitboard() | Sq::H7.bitboard();
    assert_eq!(h8_attacks, expected_h8);
    
    // Edge of board: A4
    let a4_attacks = king_attacks(Sq::A4);
    let expected_a4 = Sq::A5.bitboard() | Sq::B5.bitboard()
                    | Sq::B4.bitboard() 
                    | Sq::A3.bitboard() | Sq::B3.bitboard();
    assert_eq!(a4_attacks, expected_a4);
}

#[test]
fn test_bishop_xray_attacks() {
    // X-ray attacks don't stop at blockers, they are precomputed rays to the edge.
    // Equivalent to all diagonal rays from the square.
    // From E4, diagonals are D5-A8, F5-H7, F3-H1, D3-B1
    let e4_attacks = bishop_xray_attacks(Sq::E4);
    let expected = lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::NorthWest, Sq::E4)
                 | lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::NorthEast, Sq::E4)
                 | lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::SouthWest, Sq::E4)
                 | lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::SouthEast, Sq::E4);
    
    assert_eq!(e4_attacks, expected);

    // Test a specific corner: A1
    let a1_attacks = bishop_xray_attacks(Sq::A1);
    let expected_a1 = lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::NorthEast, Sq::A1);
    assert_eq!(a1_attacks, expected_a1);
}

#[test]
fn test_rook_xray_attacks() {
    // X-ray attacks for Rook are orthogonal rays to the edge.
    let e4_attacks = rook_xray_attacks(Sq::E4);
    let expected = lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::North, Sq::E4)
                 | lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::South, Sq::E4)
                 | lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::East, Sq::E4)
                 | lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::West, Sq::E4);
    
    assert_eq!(e4_attacks, expected);

    // Ensure it correctly masks out exactly the file and rank minus the square itself
    let rank_e4 = lucky_chess::bitboard::RANK_4;
    let file_e4 = lucky_chess::bitboard::FILE_E;
    assert_eq!(e4_attacks, (rank_e4 | file_e4) ^ Sq::E4.bitboard());
    
    // Check corner A1
    let a1_attacks = rook_xray_attacks(Sq::A1);
    let expected_a1 = lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::North, Sq::A1)
                    | lucky_chess::bitboard::bb_from_dir(lucky_chess::types::Dir::East, Sq::A1);
    assert_eq!(a1_attacks, expected_a1);
}
