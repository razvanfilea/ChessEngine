use chess_base::bitboard::*;
use chess_base::prelude::*;

#[test]
fn test_file_masks() {
    assert_eq!(FILE_A, 0x0101_0101_0101_0101);
    assert_eq!(FILE_H, 0x8080_8080_8080_8080);
    // Spot check intersections
    assert_eq!(FILE_A & FILE_H, 0);
    assert_eq!(FILE_A & Sq::A1.bitboard(), Sq::A1.bitboard());
    assert_eq!(FILE_A & Sq::A8.bitboard(), Sq::A8.bitboard());
    assert_eq!(FILE_H & Sq::H1.bitboard(), Sq::H1.bitboard());
    assert_eq!(FILE_H & Sq::H8.bitboard(), Sq::H8.bitboard());
}

#[test]
fn test_rank_masks() {
    assert_eq!(RANK_1, 0x0000_0000_0000_00FF);
    assert_eq!(RANK_8, 0xFF00_0000_0000_0000);
    // Spot check intersections
    assert_eq!(RANK_1 & RANK_8, 0);
    assert_eq!(RANK_1 & Sq::A1.bitboard(), Sq::A1.bitboard());
    assert_eq!(RANK_8 & Sq::A8.bitboard(), Sq::A8.bitboard());
}

#[test]
fn test_shifts() {
    let a1 = Sq::A1.bitboard();

    // North
    assert_eq!(sh_north(a1), Sq::A2.bitboard());
    assert_eq!(sh_north(Sq::A8.bitboard()), 0);

    // South
    assert_eq!(sh_south(Sq::A2.bitboard()), a1);
    assert_eq!(sh_south(a1), 0);

    // East
    assert_eq!(sh_east(a1), Sq::B1.bitboard());
    assert_eq!(sh_east(Sq::H1.bitboard()), 0); // Wrap around prevented

    // West
    assert_eq!(sh_west(Sq::B1.bitboard()), a1);
    assert_eq!(sh_west(a1), 0); // Wrap around prevented

    // NorthEast
    assert_eq!(sh_north_east(a1), Sq::B2.bitboard());
    assert_eq!(sh_north_east(Sq::H8.bitboard()), 0);
    assert_eq!(sh_north_east(Sq::H1.bitboard()), 0);

    // NorthWest
    assert_eq!(sh_north_west(Sq::B1.bitboard()), Sq::A2.bitboard());
    assert_eq!(sh_north_west(a1), 0);
    assert_eq!(sh_north_west(Sq::A8.bitboard()), 0);

    // SouthEast
    assert_eq!(sh_south_east(Sq::A2.bitboard()), Sq::B1.bitboard());
    assert_eq!(sh_south_east(Sq::H1.bitboard()), 0);
    assert_eq!(sh_south_east(Sq::H8.bitboard()), 0);

    // SouthWest
    assert_eq!(sh_south_west(Sq::B2.bitboard()), a1);
    assert_eq!(sh_south_west(a1), 0);
    assert_eq!(sh_south_west(Sq::A8.bitboard()), 0);

    // NorthNorth
    assert_eq!(sh_north_north(a1), Sq::A3.bitboard());
    assert_eq!(sh_north_north(Sq::A7.bitboard()), 0);
    assert_eq!(sh_north_north(Sq::A8.bitboard()), 0);

    // SouthSouth
    assert_eq!(sh_south_south(Sq::A3.bitboard()), a1);
    assert_eq!(sh_south_south(Sq::A2.bitboard()), 0);
    assert_eq!(sh_south_south(Sq::A1.bitboard()), 0);
}

#[test]
fn test_sh_dir() {
    let a1 = Sq::A1.bitboard();
    assert_eq!(sh_dir(Dir::North, a1), sh_north(a1));
    assert_eq!(
        sh_dir(Dir::South, Sq::A2.bitboard()),
        sh_south(Sq::A2.bitboard())
    );
    assert_eq!(sh_dir(Dir::East, a1), sh_east(a1));
    assert_eq!(
        sh_dir(Dir::West, Sq::B1.bitboard()),
        sh_west(Sq::B1.bitboard())
    );
    assert_eq!(sh_dir(Dir::NorthEast, a1), sh_north_east(a1));
    assert_eq!(
        sh_dir(Dir::NorthWest, Sq::B1.bitboard()),
        sh_north_west(Sq::B1.bitboard())
    );
    assert_eq!(
        sh_dir(Dir::SouthEast, Sq::A2.bitboard()),
        sh_south_east(Sq::A2.bitboard())
    );
    assert_eq!(
        sh_dir(Dir::SouthWest, Sq::B2.bitboard()),
        sh_south_west(Sq::B2.bitboard())
    );
}

#[test]
fn test_bb_from_dir() {
    // Ray from A1 going North should be A2, A3, A4, A5, A6, A7, A8
    let a1_north = bb_from_dir(Dir::North, Sq::A1);
    let expected = Sq::A2.bitboard()
        | Sq::A3.bitboard()
        | Sq::A4.bitboard()
        | Sq::A5.bitboard()
        | Sq::A6.bitboard()
        | Sq::A7.bitboard()
        | Sq::A8.bitboard();
    assert_eq!(a1_north, expected);

    // Ray from H8 going East should be 0 (no squares east of H-file)
    let h8_east = bb_from_dir(Dir::East, Sq::H8);
    assert_eq!(h8_east, 0);

    // Ray from E4 going SouthWest -> D3, C2, B1
    let e4_sw = bb_from_dir(Dir::SouthWest, Sq::E4);
    let expected_sw = Sq::D3.bitboard() | Sq::C2.bitboard() | Sq::B1.bitboard();
    assert_eq!(e4_sw, expected_sw);
}

#[test]
fn test_bb_between() {
    // Between A1 and A4 -> A2, A3
    let a1_a4 = bb_between(Sq::A1, Sq::A4);
    let expected = Sq::A2.bitboard() | Sq::A3.bitboard();
    assert_eq!(a1_a4, expected);

    // Between A4 and A1 -> same
    assert_eq!(bb_between(Sq::A4, Sq::A1), expected);

    // Between A1 and H8 -> B2, C3, D4, E5, F6, G7
    let a1_h8 = bb_between(Sq::A1, Sq::H8);
    let expected_diag = Sq::B2.bitboard()
        | Sq::C3.bitboard()
        | Sq::D4.bitboard()
        | Sq::E5.bitboard()
        | Sq::F6.bitboard()
        | Sq::G7.bitboard();
    assert_eq!(a1_h8, expected_diag);

    // Not on same ray -> 0
    assert_eq!(bb_between(Sq::A1, Sq::B3), 0);

    // Adjacent -> 0
    assert_eq!(bb_between(Sq::A1, Sq::A2), 0);
    assert_eq!(bb_between(Sq::A1, Sq::B2), 0);

    // Same square -> 0
    assert_eq!(bb_between(Sq::A1, Sq::A1), 0);
}

#[test]
fn test_bb_rank_and_file() {
    assert_eq!(bb_rank(0), RANK_1);
    assert_eq!(bb_file(0), FILE_A);
}

#[test]
fn test_bb_properties() {
    assert!(bb_several(3));
    assert!(!bb_several(2));
    assert!(!bb_several(0));

    assert!(!bb_only_one(3));
    assert!(bb_only_one(2));
    assert!(!bb_only_one(0));
}

#[test]
fn test_bb_scan() {
    assert_eq!(unsafe { bb_scan_forward(2) }, Sq::B1);
    assert_eq!(unsafe { bb_scan_reverse(2) }, Sq::B1);
    assert_eq!(bb_scan_forward_opt(2), Some(Sq::B1));
    assert_eq!(bb_scan_reverse_opt(2), Some(Sq::B1));
    assert_eq!(bb_scan_forward_opt(0), None);
    assert_eq!(bb_scan_reverse_opt(0), None);

    let mut bb = 3;
    let sq = unsafe { bb_pop_lsb(&mut bb) };
    assert_eq!(sq, Sq::A1);
    assert_eq!(bb, 2);

    let sq_opt = bb_pop_lsb_opt(&mut bb);
    assert_eq!(sq_opt, Some(Sq::B1));
    assert_eq!(bb, 0);
    assert_eq!(bb_pop_lsb_opt(&mut bb), None);
}

#[test]
fn test_bb_get_edge_filter() {
    let sq = Sq::B2;
    let filter = bb_get_edge_filter(sq);
    assert_eq!(filter, RANK_1 | RANK_8 | FILE_A | FILE_H);

    let sq_a1 = Sq::A1;
    let filter_a1 = bb_get_edge_filter(sq_a1);
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
    let empty = generate_rook_attacks(sq, 0);
    let expected_empty = (FILE_E | RANK_4) ^ sq.bitboard();
    assert_eq!(empty, expected_empty);

    let blockers = Sq::E2.bitboard() | Sq::E7.bitboard() | Sq::C4.bitboard() | Sq::G4.bitboard();
    let attacks = generate_rook_attacks(sq, blockers);
    let expected_attacks = Sq::E5.bitboard() | Sq::E6.bitboard() | Sq::E7.bitboard() |
        Sq::E3.bitboard() | Sq::E2.bitboard() |
        Sq::F4.bitboard() | Sq::G4.bitboard() |
        Sq::D4.bitboard() | Sq::C4.bitboard();
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

    let blockers = Sq::F6.bitboard() | Sq::B6.bitboard() | Sq::F2.bitboard() | Sq::B2.bitboard();
    let attacks = generate_bishop_attacks(sq, blockers);
    let expected_attacks = Sq::E5.bitboard() | Sq::F6.bitboard() |
        Sq::C5.bitboard() | Sq::B6.bitboard() |
        Sq::E3.bitboard() | Sq::F2.bitboard() |
        Sq::C3.bitboard() | Sq::B2.bitboard();
    assert_eq!(attacks, expected_attacks);
}
