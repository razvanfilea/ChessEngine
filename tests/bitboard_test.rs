use lucky_chess::bitboard::*;
use lucky_chess::types::Dir;
use lucky_chess::square::Sq;

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
    let (sq, new_bb) = bb_pop_lsb(bb);
    assert_eq!(sq.as_index(), 0);
    assert_eq!(new_bb, 2);
}

#[test]
fn test_bb_tables() {
    let sq1 = Sq::from_raw(0); // A1
    let sq2 = Sq::from_raw(2); // C1
    
    let ray = bb_from_dir(Dir::East, sq1);
    assert_ne!(ray, 0);
    
    let between = bb_between(sq1, sq2);
    assert_eq!(between, 1 << 1); // B1
    
    let line = bb_line(sq1, sq2);
    assert_eq!(line & (1 << 0), 1 << 0);
    assert_eq!(line & (1 << 1), 1 << 1);
    assert_eq!(line & (1 << 2), 1 << 2);
}

#[test]
fn test_bb_get_edge_filter() {
    let sq = Sq::from_raw(9); // B2
    let filter = bb_get_edge_filter(sq);
    assert_ne!(filter, 0);
}

#[test]
fn test_bb_generate_ray_attacks() {
    let sq = Sq::from_raw(0); // A1
    let attacks = bb_generate_ray_attacks(sq, 0, Dir::North);
    assert_ne!(attacks, 0);
}
