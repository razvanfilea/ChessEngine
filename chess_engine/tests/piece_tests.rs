use chess_core::prelude::*;

#[test]
fn test_colored_piece_new() {
    let cp = ColoredPiece::new(Piece::Knight, Color::White);
    assert_eq!(cp.piece(), Piece::Knight);
    assert_eq!(cp.color(), Color::White);
}

#[test]
fn test_colored_piece_parse() {
    assert_eq!(
        ColoredPiece::parse('P'),
        Some(ColoredPiece::new(Piece::Pawn, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('p'),
        Some(ColoredPiece::new(Piece::Pawn, Color::Black))
    );
    assert_eq!(
        ColoredPiece::parse('R'),
        Some(ColoredPiece::new(Piece::Rook, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('r'),
        Some(ColoredPiece::new(Piece::Rook, Color::Black))
    );
    assert_eq!(
        ColoredPiece::parse('N'),
        Some(ColoredPiece::new(Piece::Knight, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('n'),
        Some(ColoredPiece::new(Piece::Knight, Color::Black))
    );
    assert_eq!(
        ColoredPiece::parse('B'),
        Some(ColoredPiece::new(Piece::Bishop, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('b'),
        Some(ColoredPiece::new(Piece::Bishop, Color::Black))
    );
    assert_eq!(
        ColoredPiece::parse('Q'),
        Some(ColoredPiece::new(Piece::Queen, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('q'),
        Some(ColoredPiece::new(Piece::Queen, Color::Black))
    );
    assert_eq!(
        ColoredPiece::parse('K'),
        Some(ColoredPiece::new(Piece::King, Color::White))
    );
    assert_eq!(
        ColoredPiece::parse('k'),
        Some(ColoredPiece::new(Piece::King, Color::Black))
    );

    assert_eq!(ColoredPiece::parse('X'), None);
}

#[test]
fn test_colored_piece_eq_pieces() {
    let cp = ColoredPiece::new(Piece::Rook, Color::White);
    assert!(cp == Piece::Rook);
    assert!(cp != Piece::Pawn);
}

#[test]
fn test_colored_piece_roundtrip_all() {
    let pieces = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];
    let colors = [Color::White, Color::Black];
    for &piece in &pieces {
        for &color in &colors {
            let cp = ColoredPiece::new(piece, color);
            assert_eq!(cp.piece(), piece);
            assert_eq!(cp.color(), color);
        }
    }
}
