use chess_base::prelude::*;

#[test]
fn test_colored_piece_new() {
    let cp = ColoredPiece::new(Pieces::Knight, Color::White);
    assert_eq!(cp.piece(), Pieces::Knight);
    assert_eq!(cp.color(), Color::White);
}

#[test]
fn test_colored_piece_parse() {
    assert_eq!(
        ColoredPiece::parse('P'),
        Some(ColoredPiece::new(Pieces::Pawn, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('p'),
        Some(ColoredPiece::new(Pieces::Pawn, Color::Black))
    );
    assert_eq!(
        ColoredPiece::parse('R'),
        Some(ColoredPiece::new(Pieces::Rook, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('r'),
        Some(ColoredPiece::new(Pieces::Rook, Color::Black))
    );
    assert_eq!(
        ColoredPiece::parse('N'),
        Some(ColoredPiece::new(Pieces::Knight, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('n'),
        Some(ColoredPiece::new(Pieces::Knight, Color::Black))
    );
    // Note: Bischop is the enum variant used in the code
    assert_eq!(
        ColoredPiece::parse('B'),
        Some(ColoredPiece::new(Pieces::Bischop, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('b'),
        Some(ColoredPiece::new(Pieces::Bischop, Color::Black))
    );
    assert_eq!(
        ColoredPiece::parse('Q'),
        Some(ColoredPiece::new(Pieces::Queen, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('q'),
        Some(ColoredPiece::new(Pieces::Queen, Color::Black))
    );
    assert_eq!(
        ColoredPiece::parse('K'),
        Some(ColoredPiece::new(Pieces::King, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('k'),
        Some(ColoredPiece::new(Pieces::King, Color::Black))
    );

    assert_eq!(ColoredPiece::parse('X'), None);
}

#[test]
fn test_colored_piece_eq_pieces() {
    let cp = ColoredPiece::new(Pieces::Rook, Color::White);
    assert!(cp == Pieces::Rook);
    assert!(cp != Pieces::Pawn);
}
