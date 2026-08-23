use chess_base::prelude::*;

#[test]
fn test_sq_constants() {
    assert_eq!(Sq::NB, 64);
    assert_eq!(Sq::NONE.as_u8(), 64);
    assert_eq!(Sq::VALID_SQUARES_MASK, 0b00111111);

    assert_eq!(Sq::A1.as_u8(), 0);
    assert_eq!(Sq::B1.as_u8(), 1);
    assert_eq!(Sq::H1.as_u8(), 7);
    assert_eq!(Sq::A2.as_u8(), 8);
    assert_eq!(Sq::H8.as_u8(), 63);
}

#[test]
fn test_sq_new() {
    // Happy path: valid squares
    assert_eq!(Sq::new(0, 0), Sq::A1);
    assert_eq!(Sq::new(7, 0), Sq::H1);
    assert_eq!(Sq::new(0, 7), Sq::A8);
    assert_eq!(Sq::new(7, 7), Sq::H8);
    assert_eq!(Sq::new(4, 3), Sq::E4); // file 4(e), rank 3(4)

    // Edge cases / Error Handling: out of bounds
    // Implementation uses: ((file | rank) & !7) == 0.
    assert_eq!(Sq::new(8, 0), Sq::NONE);
    assert_eq!(Sq::new(0, 8), Sq::NONE);
    assert_eq!(Sq::new(8, 8), Sq::NONE);
    assert_eq!(Sq::new(255, 255), Sq::NONE);
}

#[test]
fn test_sq_from_raw() {
    assert_eq!(Sq::from_raw(0), Sq::A1);
    assert_eq!(Sq::from_raw(63), Sq::H8);

    // Edge cases
    assert_eq!(Sq::from_raw(64), Sq::NONE);
    assert_eq!(Sq::from_raw(255), Sq::NONE);
}

#[test]
fn test_sq_from_raw_unchecked() {
    // Happy path: Safety condition val <= 64 is met.
    unsafe {
        assert_eq!(Sq::from_raw_unchecked(0), Sq::A1);
        assert_eq!(Sq::from_raw_unchecked(63), Sq::H8);
        assert_eq!(Sq::from_raw_unchecked(64), Sq::NONE);
    }
}

#[test]
fn test_sq_as_u8_and_index() {
    assert_eq!(Sq::A1.as_u8(), 0);
    assert_eq!(Sq::H8.as_u8(), 63);
    assert_eq!(Sq::NONE.as_u8(), 64);

    assert_eq!(Sq::A1.as_index(), 0);
    assert_eq!(Sq::H8.as_index(), 63);
    assert_eq!(Sq::NONE.as_index(), 64);
}

#[test]
fn test_sq_file_and_rank() {
    assert_eq!(Sq::A1.file(), 0);
    assert_eq!(Sq::A1.rank(), 0);

    assert_eq!(Sq::H8.file(), 7);
    assert_eq!(Sq::H8.rank(), 7);

    assert_eq!(Sq::E4.file(), 4);
    assert_eq!(Sq::E4.rank(), 3);

    // NONE behavior: 64 is file 0, rank 8.
    assert_eq!(Sq::NONE.file(), 0);
    assert_eq!(Sq::NONE.rank(), 8);
}

#[test]
fn test_sq_is_valid_and_is_none() {
    assert!(Sq::A1.is_valid());
    assert!(!Sq::A1.is_none());

    assert!(Sq::H8.is_valid());
    assert!(!Sq::H8.is_none());

    assert!(!Sq::NONE.is_valid());
    assert!(Sq::NONE.is_none());
}

#[test]
fn test_sq_bitboard() {
    assert_eq!(Sq::A1.bitboard(), 1);
    assert_eq!(Sq::B1.bitboard(), 2);
    assert_eq!(Sq::H8.bitboard(), 1 << 63);

    assert_eq!(Sq::NONE.bitboard(), 0);
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
    assert_eq!(Sq::A1.distance(Sq::B2), 1);
    assert_eq!(Sq::A1.distance(Sq::H8), 7);
    assert_eq!(Sq::A1.distance(Sq::A8), 7);
    assert_eq!(Sq::A1.distance(Sq::H1), 7);

    assert_eq!(Sq::E4.distance(Sq::E4), 0);
}

#[test]
fn test_sq_manhattan_distance() {
    assert_eq!(Sq::A1.manhattan_distance(Sq::B2), 2);
    assert_eq!(Sq::A1.manhattan_distance(Sq::H8), 14);
    assert_eq!(Sq::A1.manhattan_distance(Sq::A8), 7);
    assert_eq!(Sq::A1.manhattan_distance(Sq::H1), 7);

    assert_eq!(Sq::E4.manhattan_distance(Sq::E4), 0);
}

#[test]
fn test_sq_parse() {
    assert_eq!(Sq::parse("a1"), Sq::A1);
    assert_eq!(Sq::parse("h8"), Sq::H8);
    assert_eq!(Sq::parse("e4"), Sq::E4);

    // Error cases
    assert_eq!(Sq::parse("a"), Sq::NONE); // Too short
    assert_eq!(Sq::parse(""), Sq::NONE); // Empty
    assert_eq!(Sq::parse("i1"), Sq::NONE); // Invalid file
    assert_eq!(Sq::parse("a9"), Sq::NONE); // Invalid rank
    assert_eq!(Sq::parse("H8"), Sq::NONE); // Uppercase not supported

    // Truncates extra characters successfully
    assert_eq!(Sq::parse("a1x"), Sq::A1);
}

#[test]
fn test_sq_default() {
    assert_eq!(Sq::default(), Sq::NONE);
}

#[test]
fn test_sq_fmt_debug_and_display() {
    assert_eq!(format!("{:?}", Sq::A1), "a1");
    assert_eq!(format!("{:?}", Sq::H8), "h8");
    assert_eq!(format!("{:?}", Sq::E4), "e4");
    assert_eq!(format!("{:?}", Sq::NONE), "None");

    assert_eq!(format!("{}", Sq::A1), "a1");
    assert_eq!(format!("{}", Sq::H8), "h8");
    assert_eq!(format!("{}", Sq::E4), "e4");
    assert_eq!(format!("{}", Sq::NONE), "None");
}
