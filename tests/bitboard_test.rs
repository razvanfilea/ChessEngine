use chess_base::bitboard::*;
use chess_base::prelude::*;

#[test]
fn test_sh_functions() {
    let b = 1 << 9; // B2
    assert_eq!(sh_north(b), 1 << 17);
    assert_eq!(sh_south(b), 1 << 1);
    assert_eq!(sh_east(b), 1 << 10);
    assert_eq!(sh_west(b), 1 << 8);
    assert_eq!(sh_north_east(b), 1 << 18);
    assert_eq!(sh_north_west(b), 1 << 16);
    assert_eq!(sh_south_east(b), 1 << 2);
    assert_eq!(sh_south_west(b), 1 << 0);
    assert_eq!(sh_north_north(b), 1 << 25);

    let b3 = 1 << 17; // B3
    assert_eq!(sh_south_south(b3), 1 << 1);

    assert_eq!(sh_dir(Dir::North, b), sh_north(b));
    assert_eq!(sh_dir(Dir::South, b), sh_south(b));
    assert_eq!(sh_dir(Dir::East, b), sh_east(b));
    assert_eq!(sh_dir(Dir::West, b), sh_west(b));
    assert_eq!(sh_dir(Dir::NorthEast, b), sh_north_east(b));
    assert_eq!(sh_dir(Dir::NorthWest, b), sh_north_west(b));
    assert_eq!(sh_dir(Dir::SouthEast, b), sh_south_east(b));
    assert_eq!(sh_dir(Dir::SouthWest, b), sh_south_west(b));
}

#[test]
fn test_bb_rank_and_file() {
    assert_eq!(bb_rank(0), RANK_1);
    assert_eq!(bb_file(0), FILE_A);
}

#[test]
fn test_bb_properties() {
    assert_eq!(bb_several(3), true);
    assert_eq!(bb_several(2), false);
    assert_eq!(bb_several(0), false);

    assert_eq!(bb_only_one(3), false);
    assert_eq!(bb_only_one(2), true);
    assert_eq!(bb_only_one(0), false);
}

#[test]
fn test_bb_scan() {
    assert_eq!(bb_scan_forward(2), 1);
    // Note: bb_scan_reverse has a known bug. It is expected to fail or we can assert the buggy behavior.
    // The user said "if you find any mistakes dont touch the code instead let me know".
    // I will write the test assuming standard correct behavior, so it will fail when run.
    assert_eq!(bb_scan_reverse(2), 1);

    let mut bb = 3;
    let sq = bb_pop_lsb(&mut bb);
    assert_eq!(sq.as_index(), 0);
    assert_eq!(bb, 2);
}

#[test]
fn test_bb_tables() {
    let sq1 = Sq::from_raw(0); // A1
    let sq2 = Sq::from_raw(2); // C1

    let ray = bb_from_dir(Dir::East, sq1);
    assert_ne!(ray, 0);

    let between = bb_between(sq1, sq2);
    assert_eq!(between, 1 << 1); // B1
}

#[test]
fn test_bb_get_edge_filter() {
    let sq = Sq::B2;
    let filter = bb_get_edge_filter(sq);
    // B2 is on Rank 2, File B.
    // Edge filter should include Rank 1, Rank 8, File A, File H.
    assert_eq!(filter, RANK_1 | RANK_8 | FILE_A | FILE_H);

    let sq_a1 = Sq::A1;
    let filter_a1 = bb_get_edge_filter(sq_a1);
    // A1 is on Rank 1, File A.
    // Filter should exclude Rank 1 and File A.
    assert_eq!(filter_a1, RANK_8 | FILE_H);
}

#[test]
fn test_bb_generate_ray_attacks() {
    let sq = Sq::D4;
    // North ray from D4 is D5, D6, D7, D8
    let empty_attacks = bb_generate_ray_attacks(sq, 0, Dir::North);
    assert_eq!(
        empty_attacks,
        Sq::D5.bitboard() | Sq::D6.bitboard() | Sq::D7.bitboard() | Sq::D8.bitboard()
    );

    // Blocker on D6
    let blocked_attacks = bb_generate_ray_attacks(sq, Sq::D6.bitboard(), Dir::North);
    // Should include the blocker but not squares beyond it
    assert_eq!(blocked_attacks, Sq::D5.bitboard() | Sq::D6.bitboard());

    // Blocker on D4 itself shouldn't matter since ray starts after D4
    let self_blocked = bb_generate_ray_attacks(sq, Sq::D4.bitboard(), Dir::North);
    assert_eq!(self_blocked, empty_attacks);
}

#[test]
fn test_generate_rook_attacks() {
    let sq = Sq::E4;
    // Empty board rook attacks from E4
    let empty = generate_rook_attacks(sq, 0);
    // Should be File E (excluding E4) and Rank 4 (excluding E4)
    let expected_empty = (FILE_E | RANK_4) ^ sq.bitboard();
    assert_eq!(empty, expected_empty);

    // Blockers on E2, E7, C4, G4
    let blockers = Sq::E2.bitboard() | Sq::E7.bitboard() | Sq::C4.bitboard() | Sq::G4.bitboard();
    let attacks = generate_rook_attacks(sq, blockers);
    let expected_attacks = Sq::E5.bitboard() | Sq::E6.bitboard() | Sq::E7.bitboard() | // North
        Sq::E3.bitboard() | Sq::E2.bitboard() |                     // South
        Sq::F4.bitboard() | Sq::G4.bitboard() |                     // East
        Sq::D4.bitboard() | Sq::C4.bitboard(); // West
    assert_eq!(attacks, expected_attacks);
}

#[test]
fn test_generate_bishop_attacks() {
    let sq = Sq::D4;
    let empty = generate_bishop_attacks(sq, 0);
    let expected_empty = bb_from_dir(Dir::NorthEast, sq)
        | bb_from_dir(Dir::NorthWest, sq)
        | bb_from_dir(Dir::SouthEast, sq)
        | bb_from_dir(Dir::SouthWest, sq);
    assert_eq!(empty, expected_empty);

    // Blockers on F6, B6, F2, B2
    let blockers = Sq::F6.bitboard() | Sq::B6.bitboard() | Sq::F2.bitboard() | Sq::B2.bitboard();
    let attacks = generate_bishop_attacks(sq, blockers);
    let expected_attacks = Sq::E5.bitboard() | Sq::F6.bitboard() | // NorthEast
        Sq::C5.bitboard() | Sq::B6.bitboard() | // NorthWest
        Sq::E3.bitboard() | Sq::F2.bitboard() | // SouthEast
        Sq::C3.bitboard() | Sq::B2.bitboard(); // SouthWest
    assert_eq!(attacks, expected_attacks);
}
