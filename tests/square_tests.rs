use chess_base::prelude::*;

#[test]
fn test_sq_constants() {
    assert_eq!(Sq::NB, 64);

    assert_eq!(Sq::A1 as u8, 0);
    assert_eq!(Sq::B1 as u8, 1);
    assert_eq!(Sq::H1 as u8, 7);
    assert_eq!(Sq::A2 as u8, 8);
    assert_eq!(Sq::H8 as u8, 63);
}

#[test]
fn test_sq_new() {
    // Happy path: valid squares
    assert_eq!(Sq::new(0, 0), Some(Sq::A1));
    assert_eq!(Sq::new(7, 0), Some(Sq::H1));
    assert_eq!(Sq::new(0, 7), Some(Sq::A8));
    assert_eq!(Sq::new(7, 7), Some(Sq::H8));
    assert_eq!(Sq::new(4, 3), Some(Sq::E4)); // file 4(e), rank 3(4)

    // Edge cases / Error Handling: out of bounds
    // Implementation uses: ((file | rank) & !7) == 0.
    assert_eq!(Sq::new(8, 0), None);
    assert_eq!(Sq::new(0, 8), None);
    assert_eq!(Sq::new(8, 8), None);
    assert_eq!(Sq::new(255, 255), None);
}

#[test]
fn test_sq_from_raw() {
    assert_eq!(Sq::from_raw(0), Some(Sq::A1));
    assert_eq!(Sq::from_raw(63), Some(Sq::H8));

    // Edge cases
    assert_eq!(Sq::from_raw(64), None);
    assert_eq!(Sq::from_raw(255), None);
}

#[test]
fn test_sq_from_raw_unchecked() {
    // Happy path: Safety condition val < 64 is met.
    unsafe {
        assert_eq!(Sq::from_raw_unchecked(0), Sq::A1);
        assert_eq!(Sq::from_raw_unchecked(63), Sq::H8);
    }
}

#[test]
fn test_sq_as_index() {
    assert_eq!(Sq::A1 as u8, 0);
    assert_eq!(Sq::H8 as u8, 63);

    assert_eq!(Sq::A1 as usize, 0);
    assert_eq!(Sq::H8 as usize, 63);
}

#[test]
fn test_sq_file_and_rank() {
    assert_eq!(Sq::A1.file(), 0);
    assert_eq!(Sq::A1.rank(), 0);

    assert_eq!(Sq::H8.file(), 7);
    assert_eq!(Sq::H8.rank(), 7);

    assert_eq!(Sq::E4.file(), 4);
    assert_eq!(Sq::E4.rank(), 3);
}

#[test]
fn test_sq_bitboard() {
    assert_eq!(Sq::A1.bitboard(), 1);
    assert_eq!(Sq::B1.bitboard(), 2);
    assert_eq!(Sq::H8.bitboard(), 1 << 63);
}

#[test]
fn test_sq_distance_to_edge() {
    // A1 is at edge (file 0, rank 0)
    assert_eq!(Sq::A1.distance_to_file_edge(), 0);
    assert_eq!(Sq::A1.distance_to_rank_edge(), 0);

    // H8 is at edge (file 7, rank 7)
    assert_eq!(Sq::H8.distance_to_file_edge(), 0);
    assert_eq!(Sq::H8.distance_to_rank_edge(), 0);

    // E4 is (file 4, rank 3)
    assert_eq!(Sq::E4.distance_to_file_edge(), 3);
    assert_eq!(Sq::E4.distance_to_rank_edge(), 3);

    // D5 is (file 3, rank 4)
    assert_eq!(Sq::D5.distance_to_file_edge(), 3);
    assert_eq!(Sq::D5.distance_to_rank_edge(), 3);
}

#[test]
fn test_sq_distance() {
    // Chebyshev distance
    assert_eq!(Sq::A1.distance_to(Sq::B2), 1);
    assert_eq!(Sq::A1.distance_to(Sq::H8), 7);
    assert_eq!(Sq::A1.distance_to(Sq::A8), 7);
    assert_eq!(Sq::A1.distance_to(Sq::H1), 7);

    assert_eq!(Sq::E4.distance_to(Sq::E4), 0);
}

#[test]
fn test_sq_manhattan_distance() {
    assert_eq!(Sq::A1.manhattan_distance_to(Sq::B2), 2);
    assert_eq!(Sq::A1.manhattan_distance_to(Sq::H8), 14);
    assert_eq!(Sq::A1.manhattan_distance_to(Sq::A8), 7);
    assert_eq!(Sq::A1.manhattan_distance_to(Sq::H1), 7);

    assert_eq!(Sq::E4.manhattan_distance_to(Sq::E4), 0);
}

#[test]
fn test_sq_parse() {
    assert_eq!(Sq::parse("a1"), Some(Sq::A1));
    assert_eq!(Sq::parse("h8"), Some(Sq::H8));
    assert_eq!(Sq::parse("e4"), Some(Sq::E4));

    // Error cases
    assert_eq!(Sq::parse("a"), None); // Too short
    assert_eq!(Sq::parse(""), None); // Empty
    assert_eq!(Sq::parse("i1"), None); // Invalid file
    assert_eq!(Sq::parse("a9"), None); // Invalid rank
    assert_eq!(Sq::parse("H8"), None); // Uppercase not supported

    // Truncates extra characters successfully
    assert_eq!(Sq::parse("a1x"), Some(Sq::A1));
}

#[test]
fn test_sq_fmt_debug_and_display() {
    assert_eq!(format!("{:?}", Sq::A1), "a1");
    assert_eq!(format!("{:?}", Sq::H8), "h8");
    assert_eq!(format!("{:?}", Sq::E4), "e4");

    assert_eq!(format!("{}", Sq::A1), "a1");
    assert_eq!(format!("{}", Sq::H8), "h8");
    assert_eq!(format!("{}", Sq::E4), "e4");
}

#[test]
fn test_sq_shift() {
    assert_eq!(Sq::A1.shift(Dir::North), Some(Sq::A2));
    assert_eq!(Sq::H8.shift(Dir::South), Some(Sq::H7));
    assert_eq!(Sq::E4.shift(Dir::NorthEast), Some(Sq::F5));

    // Edges (should not wrap around)
    assert_eq!(Sq::H1.shift(Dir::East), None);
    assert_eq!(Sq::A2.shift(Dir::West), None);
    assert_eq!(Sq::H8.shift(Dir::North), None);
    assert_eq!(Sq::A1.shift(Dir::SouthWest), None);
}

#[test]
fn test_sq_shift_unchecked() {
    unsafe {
        assert_eq!(Sq::A1.shift_unchecked(Dir::North), Sq::A2);
        assert_eq!(Sq::H8.shift_unchecked(Dir::South), Sq::H7);
        assert_eq!(Sq::E4.shift_unchecked(Dir::NorthEast), Sq::F5);
    }
}
