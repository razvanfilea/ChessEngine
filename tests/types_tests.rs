use chess_base::prelude::*;

#[test]
fn test_castling_rights() {
    assert_eq!(CastlingRights::WHITE_ANY.bits(), CastlingRights::WHITE_00.bits() | CastlingRights::WHITE_000.bits());
    assert_eq!(CastlingRights::BLACK_ANY.bits(), CastlingRights::BLACK_00.bits() | CastlingRights::BLACK_000.bits());
    assert_eq!(CastlingRights::ALL.bits(), CastlingRights::WHITE_ANY.bits() | CastlingRights::BLACK_ANY.bits());
    assert_eq!(CastlingRights::default(), CastlingRights::empty());
}

#[test]
fn test_dir_opposite() {
    assert_eq!(Dir::North.opposite(), Dir::South);
    assert_eq!(Dir::South.opposite(), Dir::North);
    assert_eq!(Dir::East.opposite(), Dir::West);
    assert_eq!(Dir::West.opposite(), Dir::East);
    assert_eq!(Dir::NorthEast.opposite(), Dir::SouthWest);
    assert_eq!(Dir::NorthWest.opposite(), Dir::SouthEast);
    assert_eq!(Dir::SouthEast.opposite(), Dir::NorthWest);
    assert_eq!(Dir::SouthWest.opposite(), Dir::NorthEast);
}

#[test]
fn test_dir_is_forwards() {
    assert_eq!(Dir::North.is_forwards(), true);
    assert_eq!(Dir::East.is_forwards(), true);
    assert_eq!(Dir::NorthEast.is_forwards(), true);
    assert_eq!(Dir::NorthWest.is_forwards(), true);

    assert_eq!(Dir::South.is_forwards(), false);
    assert_eq!(Dir::West.is_forwards(), false);
    assert_eq!(Dir::SouthEast.is_forwards(), false);
    assert_eq!(Dir::SouthWest.is_forwards(), false);
}
