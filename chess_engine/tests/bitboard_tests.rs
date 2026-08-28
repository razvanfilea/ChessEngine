use chess_core::bitboard::*;
use chess_core::prelude::*;

#[test]
fn test_file_and_rank_partitions() {
    let files = [
        FILE_A, FILE_B, FILE_C, FILE_D, FILE_E, FILE_F, FILE_G, FILE_H,
    ];
    let ranks = [
        RANK_1, RANK_2, RANK_3, RANK_4, RANK_5, RANK_6, RANK_7, RANK_8,
    ];

    // Files partition the board
    let mut file_union = 0u64;
    for i in 0..8 {
        assert_eq!(files[i].count_ones(), 8);
        for j in (i + 1)..8 {
            assert_eq!(files[i] & files[j], 0, "files {i} and {j} overlap");
        }
        file_union |= files[i];
    }
    assert_eq!(file_union, !0u64);

    // Ranks partition the board
    let mut rank_union = 0u64;
    for i in 0..8 {
        assert_eq!(ranks[i].count_ones(), 8);
        for j in (i + 1)..8 {
            assert_eq!(ranks[i] & ranks[j], 0, "ranks {i} and {j} overlap");
        }
        rank_union |= ranks[i];
    }
    assert_eq!(rank_union, !0u64);

    // Light and dark squares partition the board
    assert_eq!(LIGHT_SQUARES | DARK_SQUARES, !0u64);
    assert_eq!(LIGHT_SQUARES & DARK_SQUARES, 0);
    assert_eq!(LIGHT_SQUARES.count_ones(), 32);
    assert_eq!(DARK_SQUARES.count_ones(), 32);
}

#[test]
fn test_bb_rank_and_file() {
    for r in 0..8u8 {
        assert_eq!(bb_rank(r), RANK_1 << (8 * r));
        assert_eq!(bb_rank(r).count_ones(), 8);
    }
    for f in 0..8u8 {
        assert_eq!(bb_file(f), FILE_A << f);
        assert_eq!(bb_file(f).count_ones(), 8);
    }
}

#[test]
fn test_shifts() {
    let a1 = Sq::A1.bitboard();

    assert_eq!(sh_north(a1), Sq::A2.bitboard());
    assert_eq!(sh_north(Sq::A8.bitboard()), 0);
    assert_eq!(sh_south(Sq::A2.bitboard()), a1);
    assert_eq!(sh_south(a1), 0);
    assert_eq!(sh_east(a1), Sq::B1.bitboard());
    assert_eq!(sh_east(Sq::H1.bitboard()), 0);
    assert_eq!(sh_west(Sq::B1.bitboard()), a1);
    assert_eq!(sh_west(a1), 0);
    assert_eq!(sh_north_east(a1), Sq::B2.bitboard());
    assert_eq!(sh_north_east(Sq::H8.bitboard()), 0);
    assert_eq!(sh_north_east(Sq::H1.bitboard()), 0);
    assert_eq!(sh_north_west(Sq::B1.bitboard()), Sq::A2.bitboard());
    assert_eq!(sh_north_west(a1), 0);
    assert_eq!(sh_north_west(Sq::A8.bitboard()), 0);
    assert_eq!(sh_south_east(Sq::A2.bitboard()), Sq::B1.bitboard());
    assert_eq!(sh_south_east(Sq::H1.bitboard()), 0);
    assert_eq!(sh_south_east(Sq::H8.bitboard()), 0);
    assert_eq!(sh_south_west(Sq::B2.bitboard()), a1);
    assert_eq!(sh_south_west(a1), 0);
    assert_eq!(sh_south_west(Sq::A8.bitboard()), 0);
    assert_eq!(sh_north_north(a1), Sq::A3.bitboard());
    assert_eq!(sh_north_north(Sq::A7.bitboard()), 0);
    assert_eq!(sh_north_north(Sq::A8.bitboard()), 0);
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
    let a1_north = bb_from_dir(Dir::North, Sq::A1);
    let expected = Sq::A2.bitboard()
        | Sq::A3.bitboard()
        | Sq::A4.bitboard()
        | Sq::A5.bitboard()
        | Sq::A6.bitboard()
        | Sq::A7.bitboard()
        | Sq::A8.bitboard();
    assert_eq!(a1_north, expected);

    let h8_east = bb_from_dir(Dir::East, Sq::H8);
    assert_eq!(h8_east, 0);

    let e4_sw = bb_from_dir(Dir::SouthWest, Sq::E4);
    let expected_sw = Sq::D3.bitboard() | Sq::C2.bitboard() | Sq::B1.bitboard();
    assert_eq!(e4_sw, expected_sw);
}

#[test]
fn test_bb_between() {
    let a1_a4 = bb_between(Sq::A1, Sq::A4);
    let expected = Sq::A2.bitboard() | Sq::A3.bitboard();
    assert_eq!(a1_a4, expected);
    assert_eq!(bb_between(Sq::A4, Sq::A1), expected);

    let a1_h8 = bb_between(Sq::A1, Sq::H8);
    let expected_diag = Sq::B2.bitboard()
        | Sq::C3.bitboard()
        | Sq::D4.bitboard()
        | Sq::E5.bitboard()
        | Sq::F6.bitboard()
        | Sq::G7.bitboard();
    assert_eq!(a1_h8, expected_diag);

    assert_eq!(bb_between(Sq::A1, Sq::B3), 0);
    assert_eq!(bb_between(Sq::A1, Sq::A2), 0);
    assert_eq!(bb_between(Sq::A1, Sq::B2), 0);
    assert_eq!(bb_between(Sq::A1, Sq::A1), 0);
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
    assert_eq!(unsafe { bb_lsb(2) }, Sq::B1);
    assert_eq!(unsafe { bb_msb(2) }, Sq::B1);
    assert_eq!(bb_lsb_opt(2), Some(Sq::B1));
    assert_eq!(bb_msb_opt(2), Some(Sq::B1));
    assert_eq!(bb_lsb_opt(0), None);
    assert_eq!(bb_msb_opt(0), None);

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
    let empty_attacks = bb_generate_ray_attacks(sq, 0, Dir::North);
    assert_eq!(
        empty_attacks,
        Sq::D5.bitboard() | Sq::D6.bitboard() | Sq::D7.bitboard() | Sq::D8.bitboard()
    );

    let blocked_attacks = bb_generate_ray_attacks(sq, Sq::D6.bitboard(), Dir::North);
    assert_eq!(blocked_attacks, Sq::D5.bitboard() | Sq::D6.bitboard());

    let self_blocked = bb_generate_ray_attacks(sq, Sq::D4.bitboard(), Dir::North);
    assert_eq!(self_blocked, empty_attacks);
}

#[test]
fn test_bb_rook_attacks() {
    let sq = Sq::E4;
    let empty = bb_rook_attacks(sq, 0);
    let expected_empty = (FILE_E | RANK_4) ^ sq.bitboard();
    assert_eq!(empty, expected_empty);

    let blockers = Sq::E2.bitboard() | Sq::E7.bitboard() | Sq::C4.bitboard() | Sq::G4.bitboard();
    let attacks = bb_rook_attacks(sq, blockers);
    let expected_attacks = Sq::E5.bitboard()
        | Sq::E6.bitboard()
        | Sq::E7.bitboard()
        | Sq::E3.bitboard()
        | Sq::E2.bitboard()
        | Sq::F4.bitboard()
        | Sq::G4.bitboard()
        | Sq::D4.bitboard()
        | Sq::C4.bitboard();
    assert_eq!(attacks, expected_attacks);
}

#[test]
fn test_bb_bishop_attacks() {
    let sq = Sq::D4;
    let empty = bb_bishop_attacks(sq, 0);
    let expected_empty = bb_from_dir(Dir::NorthEast, sq)
        | bb_from_dir(Dir::NorthWest, sq)
        | bb_from_dir(Dir::SouthEast, sq)
        | bb_from_dir(Dir::SouthWest, sq);
    assert_eq!(empty, expected_empty);

    let blockers = Sq::F6.bitboard() | Sq::B6.bitboard() | Sq::F2.bitboard() | Sq::B2.bitboard();
    let attacks = bb_bishop_attacks(sq, blockers);
    let expected_attacks = Sq::E5.bitboard()
        | Sq::F6.bitboard()
        | Sq::C5.bitboard()
        | Sq::B6.bitboard()
        | Sq::E3.bitboard()
        | Sq::F2.bitboard()
        | Sq::C3.bitboard()
        | Sq::B2.bitboard();
    assert_eq!(attacks, expected_attacks);
}
